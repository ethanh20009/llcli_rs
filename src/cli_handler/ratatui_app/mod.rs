use ratatui::{
    layout::Margin,
    prelude::StatefulWidget,
    style::{Color, Modifier},
    text::Span,
    widgets::{ListState, Wrap},
};

use event_handler::EventHandler;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect, Size},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Padding, Paragraph, Widget},
};
use tui_scrollview::ScrollViewState;
use tui_textarea::TextArea;

use crate::provider::{ChatHistoryItem, ChatRole, Provider};

mod event_handler;
mod input;
mod state_handling;
mod tool_list_popover;

#[derive(Debug)]
pub struct App<'a, 't> {
    provider: &'a mut Provider,
    last_added_index: Option<usize>,
    event_handler: event_handler::EventHandler,
    scrollview_state: ScrollViewState,
    textarea: TextArea<'t>,
    exit: bool,
    selected_zone: SelectedZone,
    popover: Option<Popover>,
    llm_tool_options_state: ListState,
    generating: bool,
    scrolling_up: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectedZone {
    ChatHistory,
    TextInput,
}

#[derive(Debug, Clone, Copy)]
enum Popover {
    LlmToolList,
}

impl<'a, 't> App<'a, 't> {
    pub fn new(provider: &'a mut Provider) -> Self {
        Self {
            provider,
            event_handler: EventHandler::new(),
            exit: false,
            textarea: Self::create_chat_input(),
            selected_zone: SelectedZone::TextInput,
            scrollview_state: ScrollViewState::default(),
            generating: false,
            last_added_index: None,
            popover: None,
            llm_tool_options_state: ListState::default().with_selected(Some(0)),
            scrolling_up: false,
        }
    }

    fn create_chat_input() -> TextArea<'t> {
        let textarea = TextArea::default();
        textarea
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.exit {
            let event = self.event_handler.next().await?;
            self.handle_event(event)?;
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).split(area);

        let scrollview_area = layout[0].inner(Margin::new(1, 1));
        let mut scrollview = tui_scrollview::ScrollView::new(Size::new(
            scrollview_area.width,
            self.count_total_height(scrollview_area.width),
        ))
        .horizontal_scrollbar_visibility(tui_scrollview::ScrollbarVisibility::Never);
        self.render_widgets_into_scrollview(scrollview.buf_mut());

        let buf = frame.buffer_mut();

        let instructions = Line::from(vec![
            Span::from(" Quit "),
            Span::from("<Ctrl-c>").fg(Color::Blue),
            Span::from(" Scroll Up "),
            Span::from("<Up Arrow>").fg(Color::Blue),
            Span::from(" Scroll Down "),
            Span::from("<Down Arrow>").fg(Color::Blue),
        ]);

        let scrollview_selected = self.selected_zone == SelectedZone::ChatHistory;
        Self::build_block(scrollview_selected)
            .title("History")
            .title_bottom(instructions)
            .render(layout[0], buf);
        scrollview.render(scrollview_area, buf, &mut self.scrollview_state);

        self.draw_text_area_widget(buf, layout[1]);

        if let Some(popover) = self.popover {
            match popover {
                Popover::LlmToolList => self.llm_options_popup(area, frame),
            }
        }
    }

    fn count_total_height(&self, term_width: u16) -> u16 {
        self.provider
            .get_history()
            .iter()
            .map(|item| match item {
                ChatHistoryItem::FileUpload(_) => 3,
                ChatHistoryItem::Chat(chat) => {
                    Self::count_wrapped_lines(&chat.text, term_width) + 2
                }
            })
            .reduce(|acc, item| acc + item)
            .unwrap_or(0)
    }

    fn count_wrapped_lines(text: &str, width: u16) -> u16 {
        let mut line_count = 0;
        for line in text.lines() {
            let mut current_line_length = 0;
            for word in line.split_whitespace() {
                let word_len = word.len() as u16;
                if current_line_length + word_len + 1 > width {
                    line_count += 1;
                    current_line_length = word_len;
                } else {
                    current_line_length += word_len + 1;
                }
            }
            line_count += 1; // For the current line
        }
        line_count
    }
}

impl<'a, 't> App<'a, 't> {
    fn draw_text_area_widget(&mut self, buf: &mut Buffer, area: Rect) {
        let instructions = Line::from(vec![" Submit ".into(), "<C-S>".blue().bold()]);
        self.textarea.set_block(
            Self::build_block(self.selected_zone == SelectedZone::TextInput)
                .title("Prompt")
                .padding(Padding::new(2, 0, 0, 0))
                .title_bottom(instructions),
        );
        self.textarea.render(area, buf);
        Span::from(">")
            .style(Style::default().fg(Color::Green))
            .render(area.inner(Margin::new(1, 1)), buf);
    }

    fn build_block(selected: bool) -> Block<'t> {
        let border_set = if selected {
            border::THICK
        } else {
            border::PLAIN
        };

        let border_style = if selected {
            Style::new().light_blue()
        } else {
            Style::new().white()
        };

        Block::bordered()
            .border_set(border_set)
            .border_style(border_style)
            .padding(Padding::proportional(1))
    }

    fn render_widgets_into_scrollview(&self, buf: &mut Buffer) {
        let area = buf.area;
        let constraints = self.provider.get_history().iter().map(|chat| match chat {
            ChatHistoryItem::Chat(chat) => {
                Self::count_wrapped_lines(&chat.text, buf.area.width) + 2
            }
            ChatHistoryItem::FileUpload(_) => 3,
        });
        let layouts = Layout::vertical(constraints).split(area);

        for (index, chat) in self.provider.get_history().iter().enumerate() {
            self.bubble(chat).render(layouts[index], buf);
        }
    }

    fn bubble(&self, chat: &ChatHistoryItem) -> impl Widget {
        let block = Block::bordered();

        match chat {
            ChatHistoryItem::Chat(message) => {
                let text = tui_markdown::from_str(&message.text);
                let title_color = if message.role == ChatRole::User {
                    Color::Gray
                } else {
                    Color::Blue
                };
                Paragraph::new(text)
                    .block(
                        block
                            .title(message.role.display())
                            .title_style(Style::new().fg(title_color).add_modifier(Modifier::BOLD)),
                    )
                    .wrap(Wrap { trim: true })
            }
            ChatHistoryItem::FileUpload(file) => {
                Paragraph::new(file.relative_filepath.clone()).block(block.title("File upload"))
            }
        }
    }
}

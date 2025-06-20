use ratatui::prelude::StatefulWidget;
use std::{collections::HashSet, io, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use event_handler::{Event, EventHandler, LlmResponse};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect, Size},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Masked, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, Padding, Paragraph, Scrollbar, ScrollbarState, Widget,
    },
};
use tui_scrollview::{ScrollView, ScrollViewState};
use tui_textarea::TextArea;

use crate::provider::{ChatData, ChatHistoryItem, ChatRole, Provider};

mod event_handler;

#[derive(Debug)]
pub struct App<'a, 't> {
    provider: &'a mut Provider,
    event_handler: event_handler::EventHandler,
    scrollview_state: ScrollViewState,
    textarea: TextArea<'t>,
    exit: bool,
    selected_zone: SelectedZone,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectedZone {
    ChatHistory,
    TextInput,
}

enum WindowDirection {
    Up,
    Down,
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
        }
    }

    fn create_chat_input() -> TextArea<'t> {
        let title = Line::from("Chat Input".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        let mut textarea = TextArea::default();
        textarea.set_block(block);

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
        let layout = Layout::vertical(Constraint::from_ratios([(3, 4), (1, 4)])).split(area);

        let mut scrollview = tui_scrollview::ScrollView::new(Size::new(
            layout[0].width,
            self.count_total_height(layout[0].width),
        ))
        .horizontal_scrollbar_visibility(tui_scrollview::ScrollbarVisibility::Never);
        self.render_widgets_into_scrollview(scrollview.buf_mut());
        scrollview.render(layout[0], frame.buffer_mut(), &mut self.scrollview_state);

        self.draw_text_area_widget(frame, layout[1]);
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

    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    self.handle_key_event(key_event)?;
                }
            }
            Event::LlmResponse(LlmResponse::Chunk(chunk)) => self
                .provider
                .add_chat_to_context(ChatHistoryItem::Chat(ChatData::user(chunk)))?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        // Global
        match (key_event.code, key_event.modifiers, self.selected_zone) {
            (KeyCode::Esc, _, _) => self.exit(),
            (KeyCode::Char('k'), KeyModifiers::CONTROL, _) => {
                self.change_window(WindowDirection::Up)
            }
            (KeyCode::Char('j'), KeyModifiers::CONTROL, _) => {
                self.change_window(WindowDirection::Down)
            }

            (KeyCode::Char('j'), _, SelectedZone::ChatHistory) => {
                self.scroll_chat_history(WindowDirection::Down);
            }
            (KeyCode::Char('k'), _, SelectedZone::ChatHistory) => {
                self.scroll_chat_history(WindowDirection::Up);
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL, SelectedZone::TextInput) => {
                self.submit_prompt()?;
            }

            (_, _, SelectedZone::TextInput) => {
                self.textarea.input(key_event);
            }
            _ => {}
        };
        Ok(())
    }

    fn submit_prompt(&mut self) -> anyhow::Result<()> {
        self.event_handler
            .send_llm_response(event_handler::LlmResponse::Chunk(
                self.textarea.lines().join("\n"),
            ))?;
        self.textarea = TextArea::default();
        Ok(())
    }

    fn scroll_chat_history(&mut self, directon: WindowDirection) {
        match directon {
            WindowDirection::Up => {
                self.scrollview_state.scroll_up();
            }
            WindowDirection::Down => {
                self.scrollview_state.scroll_down();
            }
        };
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn change_window(&mut self, direction: WindowDirection) {
        match direction {
            WindowDirection::Up => match self.selected_zone {
                SelectedZone::ChatHistory => {
                    self.selected_zone = SelectedZone::TextInput;
                }
                SelectedZone::TextInput => {
                    self.selected_zone = SelectedZone::ChatHistory;
                }
            },
            WindowDirection::Down => match self.selected_zone {
                SelectedZone::ChatHistory => {
                    self.selected_zone = SelectedZone::TextInput;
                }
                SelectedZone::TextInput => {
                    self.selected_zone = SelectedZone::ChatHistory;
                }
            },
        }
    }
}

impl<'a, 't> App<'a, 't> {
    fn draw_text_area_widget(&mut self, frame: &mut Frame, area: Rect) {
        let instructions = Line::from(vec![" Submit ".into(), "<C-S>".blue().bold()]);
        self.textarea.set_block(
            Self::build_block(self.selected_zone == SelectedZone::TextInput)
                .title("Prompt")
                .title_bottom(instructions),
        );
        self.textarea.render(area, frame.buffer_mut());
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
                Paragraph::new(text).block(block.title(message.role.display()))
            }
            ChatHistoryItem::FileUpload(file) => {
                Paragraph::new(file.relative_filepath.clone()).block(block.title("File upload"))
            }
        }
    }
}

use std::{collections::HashSet, io, rc::Rc};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use event_handler::{Event, EventHandler, LlmResponse};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Masked, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, Padding, Paragraph, Scrollbar, ScrollbarState, Widget,
    },
};
use tui_textarea::TextArea;

use crate::provider::{ChatData, ChatHistoryItem, ChatRole, Provider};

mod event_handler;

#[derive(Debug)]
pub struct App<'a, 't> {
    provider: &'a mut Provider,
    event_handler: event_handler::EventHandler,
    chat_hist_scroll_offset: u16,
    chat_hist_scroll_state: ScrollbarState,
    chat_hist_scroll_length: usize,
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
            chat_hist_scroll_offset: 0,
            chat_hist_scroll_state: ScrollbarState::default(),
            chat_hist_scroll_length: Default::default(),
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

        let history = self.provider.get_history();
        let bubbles: Vec<ListItem> = Self::build_chat_history_bubbles(history);
        self.chat_hist_scroll_length = bubbles.len();
        self.chat_hist_scroll_state = self
            .chat_hist_scroll_state
            .content_length(self.chat_hist_scroll_length);

        frame.render_widget(self.chat_history_widget(bubbles), layout[0]);
        frame.render_stateful_widget(
            Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight),
            layout[0].inner(Margin {
                horizontal: 1,
                vertical: 1,
            }),
            &mut self.chat_hist_scroll_state,
        );

        self.draw_text_area_widget(frame, layout[1]);
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
                self.chat_hist_scroll_offset = self.chat_hist_scroll_offset.saturating_sub(1).max(0)
            }
            WindowDirection::Down => {
                self.chat_hist_scroll_offset = self
                    .chat_hist_scroll_offset
                    .saturating_add(1)
                    .min(self.chat_hist_scroll_length as u16)
            }
        };
        self.chat_hist_scroll_state = self
            .chat_hist_scroll_state
            .position(self.chat_hist_scroll_offset as usize);
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
    fn chat_history_widget(&self, bubbles: Vec<ListItem<'a>>) -> List {
        let title = Line::from(" Chat History ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Esc> ".blue().bold()]);
        let block = Self::build_block(self.selected_zone == SelectedZone::ChatHistory)
            .title(title.centered())
            .title_bottom(instructions.centered());

        List::new(bubbles).block(block)
    }

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

    fn build_chat_history_bubbles<'b>(history: &'b Vec<ChatHistoryItem>) -> Vec<ListItem<'b>> {
        history
            .iter()
            .filter(|chat| {
                if let ChatHistoryItem::Chat(data) = chat {
                    if data.role == ChatRole::System {
                        return false;
                    }
                }
                true
            })
            .map(|chat| match chat {
                ChatHistoryItem::FileUpload(file) => ListItem::new(file.relative_filepath.clone()),
                ChatHistoryItem::Chat(chat) => {
                    let sender = match chat.role {
                        ChatRole::User => "User".to_owned(),
                        ChatRole::Model => "LLM".to_owned(),
                        ChatRole::System => "".to_owned(),
                    };
                    let mut text = Text::from(vec![Line::from(sender.clone())]);

                    chat.text
                        .lines()
                        .map(|line| Line::from(line))
                        .for_each(|line| text.push_line(line));
                    ListItem::new(text)
                }
            })
            .collect()
    }
}

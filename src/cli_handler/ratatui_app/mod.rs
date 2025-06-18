use std::{collections::HashSet, io, rc::Rc};

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Masked, Span, Text},
    widgets::{Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarState, Widget},
};
use termimad::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui_textarea::{Input, TextArea};

use crate::provider::Provider;

#[derive(Debug)]
pub struct App<'a, 't> {
    provider: &'a mut Provider,
    chat_hist_scroll_offset: u16,
    chat_hist_scroll_state: ScrollbarState,
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
            exit: false,
            textarea: Self::create_chat_input(),
            selected_zone: SelectedZone::TextInput,
            chat_hist_scroll_offset: 0,
            chat_hist_scroll_state: ScrollbarState::default(),
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

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let s =
            "Veeeeeeeeeeeeeeeery    loooooooooooooooooong   striiiiiiiiiiiiiiiiiiiiiiiiiing.   ";
        let mut long_line = s.repeat(2);
        long_line.push('\n');
        let text = vec![
            Line::from("This is a line "),
            Line::from("This is a line   ".red()),
            Line::from("This is a line".on_dark_gray()),
            Line::from("This is a longer line".crossed_out()),
            Line::from(long_line.clone()),
            Line::from("This is a line".reset()),
            Line::from(vec![
                Span::raw("Masked text: "),
                Span::styled(Masked::new("password", '*'), Style::new().fg(Color::Red)),
            ]),
            Line::from("This is a line "),
            Line::from("This is a line   ".red()),
            Line::from("This is a line".on_dark_gray()),
            Line::from("This is a longer line".crossed_out()),
            Line::from(long_line.clone()),
            Line::from("This is a line".reset()),
            Line::from(vec![
                Span::raw("Masked text: "),
                Span::styled(Masked::new("password", '*'), Style::new().fg(Color::Red)),
            ]),
        ];
        let area = frame.area();
        let layout = Layout::vertical(Constraint::from_ratios([(1, 4), (3, 4)])).split(area);
        self.chat_hist_scroll_state = self.chat_hist_scroll_state.content_length(text.len());
        frame.render_widget(self.chat_history_widget(text), layout[0]);
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

    fn handle_events(&mut self) -> io::Result<()> {
        match termimad::crossterm::event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
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

            (_, _, SelectedZone::TextInput) => {
                self.textarea.input(key_event);
            }
            _ => {}
        }
    }

    fn scroll_chat_history(&mut self, directon: WindowDirection) {
        match directon {
            WindowDirection::Up => {
                self.chat_hist_scroll_offset = self.chat_hist_scroll_offset.saturating_sub(1);
            }
            WindowDirection::Down => {
                self.chat_hist_scroll_offset = self.chat_hist_scroll_offset.saturating_add(1);
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
    fn chat_history_widget(&self, text: Vec<Line<'a>>) -> Paragraph {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Esc> ".blue().bold()]);
        let block = Self::build_block(self.selected_zone == SelectedZone::ChatHistory)
            .title(title.centered())
            .title_bottom(instructions.centered());

        Paragraph::new(text)
            .centered()
            .scroll((self.chat_hist_scroll_offset, 0))
            .block(block)
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
}

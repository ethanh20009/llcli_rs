use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use tui_textarea::TextArea;

use crate::provider::Provider;

#[derive(Debug)]
pub struct App<'a, 't> {
    provider: &'a mut Provider,
    textarea: TextArea<'t>,
    exit: bool,
}

impl<'a, 't> App<'a, 't> {
    pub fn new(provider: &'a mut Provider) -> Self {
        Self {
            provider,
            exit: false,
            textarea: Self::create_chat_input(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl<'a, 't> Widget for &App<'a, 't> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical(Constraint::from_ratios([(3, 4), (1, 4)])).split(area);
        self.render_chat_history(layout[0], buf);
        self.textarea.render(layout[1], buf);
    }
}

impl<'a, 't> App<'a, 't> {
    fn render_chat_history(&self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec!["Value: ".into()])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
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
}

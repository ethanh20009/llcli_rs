use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Popover, SelectedZone};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Input {
    ScrollUp,
    ScrollDown,
    ChangeWindowUp,
    ChangeWindowDown,
    Back,
    Quit,
    Submit,
    Toggle,
    ToggleLlmOptions,
    None,
}

impl From<(KeyEvent)> for Input {
    fn from(value: KeyEvent) -> Self {
        let key_event = value;

        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Input::Quit,
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Input::Quit,
            (KeyCode::Up, KeyModifiers::CONTROL) => Input::ChangeWindowUp,
            (KeyCode::Down, KeyModifiers::CONTROL) => Input::ChangeWindowDown,
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => Input::ChangeWindowUp,
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => Input::ChangeWindowDown,
            (KeyCode::Up, KeyModifiers::NONE) => Input::ScrollUp,
            (KeyCode::Down, KeyModifiers::NONE) => Input::ScrollDown,
            (KeyCode::Char('k'), KeyModifiers::NONE) => Input::ScrollUp,
            (KeyCode::Char('j'), KeyModifiers::NONE) => Input::ScrollDown,
            (KeyCode::Enter, _) => Input::Toggle,
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => Input::Submit,
            (KeyCode::Tab, KeyModifiers::NONE) => Input::ToggleLlmOptions,
            (KeyCode::Esc, _) => Input::Back,
            _ => Input::None,
        }
    }
}

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::SelectedZone;

#[derive(Clone, Copy, PartialEq)]
pub enum Input {
    ScrollUp,
    ScrollDown,
    ChangeWindowUp,
    ChangeWindowDown,
    Quit,
    Submit,
    TextAreaInput(KeyEvent),
    None,
}

impl From<(KeyEvent, SelectedZone)> for Input {
    fn from(value: (KeyEvent, SelectedZone)) -> Self {
        let (key_event, zone) = value;

        // Global
        let global = match (key_event.code, key_event.modifiers) {
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Input::Quit,
            (KeyCode::Char('k'), KeyModifiers::CONTROL) | (KeyCode::Up, KeyModifiers::CONTROL) => {
                Input::ChangeWindowUp
            }
            (KeyCode::Char('j'), KeyModifiers::CONTROL)
            | (KeyCode::Down, KeyModifiers::CONTROL) => Input::ChangeWindowDown,
            _ => Input::None,
        };
        if global != Input::None {
            return global;
        }

        // Zone dependent
        match (key_event.code, key_event.modifiers, zone) {
            (KeyCode::Char('q'), KeyModifiers::NONE, SelectedZone::ChatHistory) => Input::Quit,
            (KeyCode::Char('s'), KeyModifiers::CONTROL, SelectedZone::TextInput) => Input::Submit,
            (_, _, SelectedZone::TextInput) => Input::TextAreaInput(key_event),

            (KeyCode::Char('k'), KeyModifiers::NONE, SelectedZone::ChatHistory)
            | (KeyCode::Up, KeyModifiers::NONE, SelectedZone::ChatHistory) => Input::ScrollUp,
            (KeyCode::Char('j'), KeyModifiers::NONE, SelectedZone::ChatHistory)
            | (KeyCode::Down, KeyModifiers::NONE, SelectedZone::ChatHistory) => Input::ScrollDown,
            _ => Input::None,
        }
    }
}

mod chat;
mod code;

use termimad::MadSkin;

use super::file_input::FILE_INPUT_TRIGGER;
use super::{ChatCommand, Cli, CliHandler};
use super::{CommandState, Provider};

enum ChatAction {
    AddFile { path: String },
    Text(String),
    Clear,
    End,
}

impl CliHandler {
    fn get_message(&self) -> super::error::Result<ChatAction> {
        let response = inquire::Text::new("Enter message (leave blank to exit):")
            .with_help_message("Try: '#file:', '/clear'")
            .with_autocomplete(self.file_handler.clone())
            .prompt()
            .map_err(super::error::map_inquire_error)?;

        if response.contains(FILE_INPUT_TRIGGER) {
            Ok(ChatAction::AddFile {
                path: response
                    .trim_start_matches(FILE_INPUT_TRIGGER)
                    .trim()
                    .to_string(),
            })
        } else if response.trim() == "/clear" {
            Ok(ChatAction::Clear)
        } else if response == "" {
            Ok(ChatAction::End)
        } else {
            Ok(ChatAction::Text(response))
        }
    }
}

fn output_response(response: &str, state: &CommandState) {
    if state.quiet {
        print!("{}", response);
    } else {
        let skin = MadSkin::default();
        skin.print_text(format!("---\n{}\n---", response).as_str());
    }
}

fn output_file_added(path: &str) {
    let skin = MadSkin::default();
    skin.print_text(&format!("---\nFile Added: {}\n---", path));
}

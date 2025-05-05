use anyhow::Context;
use termimad::{MadSkin, print_text};

use super::file_input::{FILE_INPUT_TRIGGER, FileInputHandler};
use super::{ChatCommand, Cli, CliHandler};
use super::{CommandState, Provider};

enum ChatAction {
    AddFile { path: String },
    Text(String),
    End,
}

impl CliHandler {
    fn get_message(&self) -> super::error::Result<ChatAction> {
        let file_handler =
            FileInputHandler::new().context("Failed to construct file input handler.")?;
        let response = inquire::Text::new("Enter message (leave blank to exit):")
            .with_autocomplete(file_handler)
            .prompt()
            .map_err(super::error::map_inquire_error)?;

        if response.contains(FILE_INPUT_TRIGGER) {
            Ok(ChatAction::AddFile {
                path: response
                    .trim_start_matches(FILE_INPUT_TRIGGER)
                    .trim()
                    .to_string(),
            })
        } else if response == "" {
            Ok(ChatAction::End)
        } else {
            Ok(ChatAction::Text(response))
        }
    }
}

impl Cli {
    pub(super) async fn handle_chat(
        command: ChatCommand,
        state: &CommandState<'_>,
    ) -> anyhow::Result<()> {
        let mut llm_provider =
            Provider::new(&state.config, &state.api_key_manager, state.cli_handler);
        llm_provider.merge_tools(command.get_tools());

        match (command.message, &state.cli_handler) {
            (None, Some(handler)) => loop {
                let prompt = handler
                    .get_message()
                    .context("Failed to retrieve message from user.")?;

                match prompt {
                    ChatAction::Text(text) => {
                        let response = llm_provider
                            .complete_chat(text)
                            .await
                            .context("Failed to retrieve response from the LLM Provider")?;

                        output_response(response.as_str(), state);
                    }
                    ChatAction::AddFile { path } => {
                        llm_provider.add_chat_to_context(
                            handler
                                .file_handler
                                .chat_from_file(&path)
                                .context("Failed to add file to context.")?,
                        )?;
                        print_text(&format!("---\nFile Added: {}\n---", path));
                    }
                    ChatAction::End => return Ok(()),
                }
            },
            (message, _) => {
                let prompt = message.context("No message supplied. Use -m to pass a message.")?;

                let response = llm_provider
                    .complete_chat(prompt)
                    .await
                    .context("Failed to retrieve response from the LLM Provider")?;

                output_response(response.as_str(), state);

                Ok(())
            }
        }
    }
}

fn output_response(response: &str, state: &CommandState) {
    if state.quiet {
        println!("{}", response);
    } else {
        let skin = MadSkin::default();
        skin.print_text(format!("---\n{}\n---", response).as_str());
    }
}

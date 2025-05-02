use anyhow::Context;
use termimad::MadSkin;

use super::{ChatCommand, Cli, CliHandler};
use super::{CommandState, Provider};

impl CliHandler {
    pub fn get_message(&self) -> super::error::Result<String> {
        inquire::Text::new("Enter message (leave blank to exit):")
            .prompt()
            .map_err(super::error::map_inquire_error)
    }
}

impl Cli {
    pub(super) async fn handle_chat(
        command: ChatCommand,
        state: &CommandState<'_>,
    ) -> anyhow::Result<()> {
        let mut llm_provider =
            Provider::new(&state.config, &state.api_key_manager, state.cli_handler);

        match (command.message, &state.cli_handler) {
            (None, Some(handler)) => loop {
                let prompt = handler
                    .get_message()
                    .context("Failed to retrieve message from user.")?;
                if prompt == "" {
                    return Ok(());
                }
                let response = llm_provider
                    .complete_chat(prompt)
                    .await
                    .context("Failed to retrieve response from the LLM Provider")?;

                output_response(response.as_str(), state);
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

use anyhow::Context;

use crate::provider::{ChatData, Provider};

use super::{ChatAction, ChatCommand, Cli, CommandState, output_response};

impl Cli {
    pub(crate) async fn handle_code(
        command: ChatCommand,
        state: &CommandState<'_>,
    ) -> anyhow::Result<()> {
        let mut llm_provider =
            Provider::new(&state.config, &state.api_key_manager, state.cli_handler);
        llm_provider.merge_tools(command.get_tools());
        llm_provider.add_chat_to_context(ChatData {
            role: crate::provider::ChatRole::System,
            text: "The user is issuing a code generation command. You must only respond with the code you have generated.".to_string()
        }.into())?;

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
                    }
                    ChatAction::Clear => {
                        llm_provider.clear_history()?;
                    }
                    ChatAction::End => return Ok(()),
                }
            },
            (message, _) => {
                let prompt = message.context("No message supplied. Use -m to pass a message.")?;

                let mut response = llm_provider
                    .complete_chat(prompt)
                    .await
                    .context("Failed to retrieve response from the LLM Provider")?;

                if state.quiet {
                    response = parse_code_response(&response);
                }

                output_response(response.as_str(), state);

                Ok(())
            }
        }
    }
}

fn parse_code_response(response: &str) -> String {
    if response.starts_with("```") {
        let lines = response.split('\n').collect::<Vec<_>>();
        lines
            .into_iter()
            .filter(|line| !line.contains("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        response.to_string()
    }
}

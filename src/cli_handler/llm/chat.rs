use anyhow::Context;
use tokio_stream::StreamExt;

use super::{ChatAction, ChatCommand, Cli, output_file_added, output_response, output_seperator};
use super::{CommandState, Provider};

impl Cli {
    pub(crate) async fn handle_chat(
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
                        let mut llm_response_acc = String::new();
                        {
                            let mut response = llm_provider
                                .complete_chat_stream(text.clone())
                                .await
                                .context("Failed to retrieve response from the LLM Provider")?;
                            output_seperator(state);
                            while let Some(llm_response) = response.next().await {
                                let response = llm_response?;
                                output_response(response.as_str(), state);
                                llm_response_acc.push_str(&response);
                            }
                            output_seperator(state);
                        }
                        llm_provider.update_memory(text, llm_response_acc)?;
                    }
                    ChatAction::AddFile { path } => {
                        llm_provider.add_chat_to_context(
                            handler
                                .file_handler
                                .chat_from_file(&path)
                                .context("Failed to add file to context.")?,
                        )?;
                        output_file_added(&path);
                    }
                    ChatAction::Clear => {
                        llm_provider.clear_history()?;
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

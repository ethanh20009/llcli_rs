use anyhow::Context;

use crate::cli_handler::ratatui_app::App;

use super::{ChatAction, ChatCommand, Cli, output_file_added, output_response};
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
            (None, Some(handler)) => {
                let mut app = App::new(&mut llm_provider);
                let mut terminal = ratatui::init();
                let app_result = app.run(&mut terminal).await;
                ratatui::restore();
                app_result.context("Ratatui Terminal Error.")
            }
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

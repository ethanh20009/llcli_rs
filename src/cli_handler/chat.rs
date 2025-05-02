use anyhow::Context;
use termimad::MadSkin;

use super::{ChatCommand, Cli};
use super::{CommandState, Provider};
impl Cli {
    pub(super) async fn handle_chat(
        command: ChatCommand,
        state: &CommandState<'_>,
    ) -> anyhow::Result<()> {
        let llm_provider = Provider::new(&state.config, &state.api_key_manager, state.cli_handler);

        let prompt = match command.message {
            Some(message) => Some(message),
            None => match &state.cli_handler {
                Some(handler) => Some(handler.get_message().context("Failed to get message.")?),
                None => None,
            },
        }
        .context("No message supplied")?;

        let response = llm_provider
            .complete_chat(prompt)
            .await
            .context("Failed to retrieve response from the LLM Provider")?;

        if state.quiet {
            println!("{}", response);
        } else {
            let skin = MadSkin::default();
            skin.print_text(format!("---\n{}\n---", response.as_str()).as_str());
        }
        Ok(())
    }
}

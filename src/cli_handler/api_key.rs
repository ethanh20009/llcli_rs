use anyhow::Context;

use super::{Cli, CommandState, SetApiKeyCommand};

impl Cli {
    pub(super) fn handle_api_key(
        command: SetApiKeyCommand,
        state: &CommandState,
    ) -> anyhow::Result<()> {
        let key = command
            .key
            .or_else(|| {
                state
                    .cli_handler
                    .and_then(|handler| Some(handler.get_message()))
            })
            .context("No API key supplied")?;
        state
            .api_key_manager
            .set_api_key(&state.config.provider, &key)
            .context("Failed to set api key.")?;
        Ok(())
    }
}

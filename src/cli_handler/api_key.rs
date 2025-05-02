use anyhow::Context;

use super::{Cli, CommandState, SetApiKeyCommand};

impl Cli {
    pub(super) fn handle_api_key(
        command: SetApiKeyCommand,
        state: &CommandState,
    ) -> anyhow::Result<()> {
        let key = match command.key {
            Some(key) => Some(key),
            None => match state.cli_handler {
                Some(handler) => Some(handler.get_api_key().context("Failed to get api key")?),
                None => None,
            },
        }
        .context("No API key supplied")?;
        state
            .api_key_manager
            .set_api_key(&state.config.provider, &key)
            .context("Failed to set api key.")?;
        Ok(())
    }
}

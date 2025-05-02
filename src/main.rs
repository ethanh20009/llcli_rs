use clap::Parser;
use cli_handler::Cli;
use configuration::ConfigManager;

mod cli_handler;
mod configuration;
mod provider;

use provider::APIKeyManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ConfigManager::new();
    let api_key_manager = APIKeyManager::new();

    let cli = Cli::parse();
    let result = cli.handle_command(&config.config, &api_key_manager).await;

    result
}

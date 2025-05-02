use clap::Parser;
use cli_handler::Cli;
use configuration::ConfigManager;

mod cli_handler;
mod configuration;
mod provider;

#[tokio::main]
async fn main() {
    let config = ConfigManager::new();
    let cli = Cli::parse();
}

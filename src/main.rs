use clap::Parser;
use cli_handler::Cli;
use configuration::ConfigManager;
use tracing_appender::non_blocking::WorkerGuard;

mod cli_handler;
mod configuration;
mod provider;

use provider::APIKeyManager;

fn init_tracing() -> WorkerGuard {
    let log_file_path = std::env::temp_dir().join("llcli_debug.log");
    let file_writer = std::fs::File::create(&log_file_path).expect("Failed to create log file");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_writer);

    tracing_subscriber::fmt()
        .pretty()
        .with_thread_names(true)
        .with_max_level(tracing::Level::TRACE)
        .with_writer(non_blocking_appender)
        .init();
    guard
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = init_tracing();

    let config = ConfigManager::new();
    let api_key_manager = APIKeyManager::new();

    let cli = Cli::parse();
    let result = cli.handle_command(&config.config, &api_key_manager).await;

    result
}

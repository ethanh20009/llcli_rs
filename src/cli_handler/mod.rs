use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};

mod api_key;
mod chat;

use crate::{
    configuration::Configuration,
    provider::{APIKeyManager, Provider},
};

pub struct CliHandler;
impl CliHandler {
    pub fn get_api_key(&self) -> String {
        inquire::Text::new("Enter API key:")
            .prompt()
            .expect("Failed to retrieve API key from user")
    }

    pub fn get_message(&self) -> String {
        inquire::Text::new("Enter message:")
            .prompt()
            .expect("Failed to retrieve message from user")
    }
}

/// LLM CLI Interface for your LLM needs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    quiet: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Args, Debug)]
pub struct ChatCommand {
    /// ask a one shot message
    #[arg(short, long)]
    message: Option<String>,
}

#[derive(Args, Debug)]
pub struct SetApiKeyCommand {
    /// set api key to value
    #[arg(short, long)]
    key: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Chat(ChatCommand),
    SetApiKey(SetApiKeyCommand),
}

impl Cli {
    pub async fn handle_command(
        self,
        config: &Configuration,
        api_key_manager: &APIKeyManager,
    ) -> anyhow::Result<()> {
        let cli_handler = if self.quiet { None } else { Some(CliHandler) };
        let state = CommandState::new(cli_handler.as_ref(), config, &api_key_manager, self.quiet);
        if let Some(command) = self.command {
            match command {
                Commands::Chat(command) => Cli::handle_chat(command, &state).await,
                Commands::SetApiKey(command) => Cli::handle_api_key(command, &state),
            }
        } else {
            Err(anyhow!("No argument given. Use --help for options."))
        }
    }
}

struct CommandState<'a> {
    cli_handler: Option<&'a CliHandler>,
    config: &'a Configuration,
    api_key_manager: &'a APIKeyManager,
    quiet: bool,
}

impl<'a> CommandState<'a> {
    fn new(
        cli_handler: Option<&'a CliHandler>,
        config: &'a Configuration,
        api_key_manager: &'a APIKeyManager,
        quiet: bool,
    ) -> Self {
        Self {
            config,
            cli_handler,
            api_key_manager,
            quiet,
        }
    }
}

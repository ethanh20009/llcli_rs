use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use file_input::FileInputHandler;

mod api_key;
mod error;
mod file_input;
mod llm;

use crate::{
    configuration::Configuration,
    provider::{APIKeyManager, LLMTools, Provider},
};

pub struct CliHandler {
    file_handler: FileInputHandler,
}
impl CliHandler {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            file_handler: FileInputHandler::new()?,
        })
    }
    pub fn get_api_key(&self) -> error::Result<String> {
        inquire::Text::new("Enter API key:")
            .prompt()
            .map_err(error::map_inquire_error)
    }

    pub fn get_command(&self) -> error::Result<Commands> {
        const CHAT: &'static str = "Chat";
        const CHAT_SEARCH: &'static str = "Chat (search)";
        const APIKEY: &'static str = "Set API Key";
        let options_str: Vec<&str> = vec![CHAT, CHAT_SEARCH, APIKEY];
        let command = inquire::Select::new("Select option:", options_str)
            .prompt()
            .map_err(error::map_inquire_error)?;
        match command {
            CHAT => Ok(Commands::Chat(ChatCommand {
                message: None,
                search: bool::default(),
            })),
            APIKEY => Ok(Commands::SetApiKey(SetApiKeyCommand { key: None })),
            CHAT_SEARCH => Ok(Commands::Chat(ChatCommand {
                message: None,
                search: true,
            })),
            _ => Err(error::Error::CommandNotOption(command.to_string())),
        }
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

    #[arg(short, long)]
    search: bool,
}

impl ChatCommand {
    pub fn get_tools(&self) -> LLMTools {
        LLMTools {
            search: self.search,
            ..Default::default()
        }
    }
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
    Code(ChatCommand),
    SetApiKey(SetApiKeyCommand),
}

impl Cli {
    pub async fn handle_command(
        self,
        config: &Configuration,
        api_key_manager: &APIKeyManager,
    ) -> anyhow::Result<()> {
        let cli_handler = if self.quiet {
            None
        } else {
            Some(CliHandler::new()?)
        };
        let state = CommandState::new(cli_handler.as_ref(), config, &api_key_manager, self.quiet);

        let command = match self.command {
            Some(command) => Some(command),
            None => match &cli_handler {
                Some(handler) => Some(handler.get_command().context("Failed to get command.")?),
                None => None,
            },
        }
        .context("No argument given. Use --help for options.")?;

        let result = match command {
            Commands::Chat(command) => Cli::handle_chat(command, &state).await,
            Commands::Code(command) => Cli::handle_code(command, &state).await,
            Commands::SetApiKey(command) => Cli::handle_api_key(command, &state),
        };

        match result {
            Ok(result) => Ok(result),
            Err(err) => match err.root_cause().downcast_ref::<error::Error>() {
                Some(error::Error::Interrupted) => Ok(()),
                _ => Err(err),
            },
        }
    }
}

pub(super) struct CommandState<'a> {
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

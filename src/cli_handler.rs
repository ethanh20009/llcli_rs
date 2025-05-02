use anyhow::{Context, anyhow};
use clap::{Parser, Subcommand};

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
        inquire::Text::new("Enter message")
            .prompt()
            .expect("Failed to retrieve message from user")
    }
}

/// LLM CLI Interface for your LLM needs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Chat {
        /// ask a one shot message
        #[arg(short, long)]
        message: Option<String>,
    },
    SetApiKey {
        /// set api key to value
        #[arg(short, long)]
        key: Option<String>,
    },
}

impl Cli {
    pub async fn handle_command(
        self,
        cli_handler: &CliHandler,
        config: &Configuration,
        api_key_manager: &APIKeyManager,
    ) -> anyhow::Result<()> {
        let provider_str = config.provider.clone();
        if let Some(command) = self.command {
            match command {
                Commands::SetApiKey { key: Some(key) } => {
                    api_key_manager
                        .set_api_key(&provider_str, &key)
                        .context("Failed to set api key.")?;
                    Ok(())
                }
                Commands::SetApiKey { key: None } => {
                    let key = cli_handler.get_api_key();
                    api_key_manager
                        .set_api_key(&provider_str, &key)
                        .context("Failed to set api key.")?;
                    Ok(())
                }
                Commands::Chat { message } => {
                    let llm_provider =
                        Provider::new(&config, &api_key_manager, None, &config.provider);

                    let prompt = if let Some(message) = message {
                        message
                    } else {
                        cli_handler.get_message()
                    };
                    let response = llm_provider
                        .complete_chat(prompt)
                        .await
                        .context("Failed to retrieve response from the LLM Provider")?;
                    println!("{}", response);
                    Ok(())
                }
            }
        } else {
            Err(anyhow!("No argument given. Use --help for options."))
        }
    }
}

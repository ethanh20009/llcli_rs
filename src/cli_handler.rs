use clap::{Parser, Subcommand};

pub struct CliHandler;
impl CliHandler {
    pub fn get_api_key(&self) -> String {
        inquire::Text::new("Enter API key")
            .prompt()
            .expect("Failed to retrieve API key from user")
    }
}

/// LLM CLI Interface for your LLM needs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
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

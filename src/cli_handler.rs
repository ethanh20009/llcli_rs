use clap::Parser;

pub struct CliHandler;
impl CliHandler {
    pub fn get_api_key(&self) -> String {}
}

/// LLM CLI Interface for your LLM needs.
#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'm')]
    message: Option<String>,
}

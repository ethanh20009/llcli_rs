mod api_key_manager;
mod error;
mod gemini;

use anyhow::Context;
pub use api_key_manager::APIKeyManager;
use gemini::GeminiProvider;
use reqwest::Url;
use serde::de::DeserializeOwned;

use crate::configuration::Configuration;
use crate::{cli_handler::CliHandler, configuration::OnlineProviderOpts};
use error::{Error, Result};

struct OnlineProvider {
    api_key: String,
    url: String,
    model: String,
}

impl OnlineProvider {
    pub fn new(
        provider: &str,
        config: &OnlineProviderOpts,
        api_key_manager: &APIKeyManager,
        cli_handler: Option<&CliHandler>,
    ) -> Self {
        let api_key_result = api_key_manager.fetch_api_key(provider);
        let api_key = match (api_key_result, cli_handler) {
            (Ok(key), _) => key,
            (Err(Error::NoApiKey), Some(cli_handler)) => {
                cli_handler.get_api_key().expect("Failed to get api key")
            }
            (Err(Error::NoApiKey), None) => panic!("No api key found. Please add an api key."),
            (Err(Error::KeyFetchError(err)), _) => {
                panic!("Error fetching key from keychain. {:?}", err)
            }
        };
        Self {
            url: config.url.clone(),
            model: config.model.clone(),
            api_key,
        }
    }
}

trait ProviderImpl {
    fn provider_str() -> &'static str;
}

trait OnlineProviderImpl: ProviderImpl {
    type ProviderApiResponse: DeserializeOwned;

    fn build_chat_url(&self) -> anyhow::Result<reqwest::Url>;
    fn build_chat_body(&self, prompt: impl Into<String>) -> serde_json::Value;
    fn get_http_client(&self) -> &reqwest::Client;
    fn decode_llm_response(&self, response: Self::ProviderApiResponse) -> anyhow::Result<String>;

    async fn complete_chat(&self, prompt: String) -> anyhow::Result<String> {
        let response = self
            .get_http_client()
            .post(self.build_chat_url()?)
            .json(&self.build_chat_body(prompt))
            .send()
            .await
            .context("Request failed to LLM Provider.")?
            .json::<Self::ProviderApiResponse>()
            .await
            .context("Failed to decode LLM response into JSON")?;
        self.decode_llm_response(response)
    }
}

#[derive(derive_more::From)]
pub enum Provider {
    Gemini(GeminiProvider),
}

const GEMINI_PROVIDER: &'static str = "gemini";

impl Provider {
    pub async fn complete_chat(&self, prompt: String) -> anyhow::Result<String> {
        match self {
            Self::Gemini(prov) => prov.complete_chat(prompt).await,
        }
    }

    pub fn new(
        config: &Configuration,
        api_key_manager: &APIKeyManager,
        cli_handler: Option<&CliHandler>,
    ) -> Provider {
        match config.provider.as_str() {
            GEMINI_PROVIDER => {
                GeminiProvider::new(&config.provider_opts.gemini, api_key_manager, cli_handler)
                    .into()
            }
            _ => panic!(
                "invalid provider string reference. Recieved: {:?}",
                config.provider.as_str()
            ),
        }
    }
}

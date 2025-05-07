mod api_key_manager;
mod error;
mod gemini;

use anyhow::Context;
pub use api_key_manager::APIKeyManager;
use gemini::GeminiProvider;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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

    fn merge_tools(&mut self, tools: LLMTools);

    fn update_memory(&mut self, prompt: String, response: String) -> anyhow::Result<()>;
    fn add_chat_to_context(&mut self, chat: Chat) -> anyhow::Result<()>;
    fn clear_memory(&mut self) -> anyhow::Result<()>;
}

trait OnlineProviderImpl: ProviderImpl {
    type ProviderApiResponse: DeserializeOwned;

    fn build_chat_url(&self) -> anyhow::Result<reqwest::Url>;
    fn build_chat_body(&self, prompt: impl Into<String>) -> serde_json::Value;
    fn get_http_client(&self) -> &reqwest::Client;
    fn decode_llm_response(&self, response: Self::ProviderApiResponse) -> anyhow::Result<String>;

    async fn complete_chat(&mut self, prompt: String) -> anyhow::Result<String> {
        let response = self
            .get_http_client()
            .post(self.build_chat_url()?)
            .json(&self.build_chat_body(prompt.clone()))
            .send()
            .await
            .context("Request failed to LLM Provider.")?
            .json::<Self::ProviderApiResponse>()
            .await
            .context("Failed to decode LLM response into JSON")?;
        let decoded = self.decode_llm_response(response)?;
        self.update_memory(prompt, decoded.clone())
            .context("Failed to update memory.")?;
        Ok(decoded)
    }
}

#[derive(derive_more::From)]
pub enum Provider {
    Gemini(GeminiProvider),
}

const GEMINI_PROVIDER: &'static str = "gemini";

impl Provider {
    pub async fn complete_chat(&mut self, prompt: String) -> anyhow::Result<String> {
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
            GEMINI_PROVIDER => GeminiProvider::new(&config, api_key_manager, cli_handler).into(),
            _ => panic!(
                "invalid provider string reference. Recieved: {:?}",
                config.provider.as_str()
            ),
        }
    }

    pub fn merge_tools(&mut self, tools: LLMTools) {
        match self {
            Self::Gemini(provider) => provider.merge_tools(tools),
        }
    }

    pub fn add_chat_to_context(&mut self, chat: Chat) -> anyhow::Result<()> {
        match self {
            Self::Gemini(provider) => provider.add_chat_to_context(chat),
        }
    }

    pub fn clear_history(&mut self) -> anyhow::Result<()> {
        match self {
            Self::Gemini(provider) => provider.clear_memory(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct LLMTools {
    pub search: bool,
}

impl LLMTools {
    pub fn new(config: &Configuration) -> Self {
        LLMTools {
            search: config
                .tools
                .as_ref()
                .and_then(|tool_options| tool_options.search_default)
                .unwrap_or_default(),
        }
    }

    fn merge(&mut self, tool_flags: &LLMTools) {
        if tool_flags.search {
            self.search = true
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ChatRole {
    User,
    Model,
    System,
}

pub(crate) struct Chat {
    pub(crate) role: ChatRole,
    pub(crate) text: String,
}

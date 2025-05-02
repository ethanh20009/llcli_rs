use gemini_api_response::GeminiApiResponse;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::{cli_handler::CliHandler, configuration::GeminiProviderOpts};
use anyhow::{Context, Result};

use super::{APIKeyManager, GEMINI_PROVIDER, OnlineProvider, OnlineProviderImpl, ProviderImpl};

pub struct GeminiProvider {
    provider: OnlineProvider,
    http_client: reqwest::Client,
}

impl ProviderImpl for GeminiProvider {
    fn provider_str() -> &'static str {
        GEMINI_PROVIDER
    }
}

impl OnlineProviderImpl for GeminiProvider {
    type ProviderApiResponse = GeminiApiResponse;
    fn build_chat_url(&self) -> Url {
        reqwest::Url::parse(&self.provider.url)
            .expect(format!("Failed to parse provider url: {:?}", self.provider.url).as_str())
            .join("v1beta/models/")
            .expect("Failed to build gemini url.")
            .join(&self.provider.model)
            .expect(
                format!(
                    "Failed to build gemini model url: {:?}.",
                    self.provider.model
                )
                .as_str(),
            )
            .join(":generateContent")
            .expect("Failed to append url command.")
    }

    fn build_chat_body(&self, prompt: impl Into<String>) -> serde_json::Value {
        todo!()
    }

    fn get_http_client(&self) -> reqwest::Client {
        todo!()
    }

    fn decode_llm_response(&self, response: GeminiApiResponse) -> anyhow::Result<String> {
        let text = response
            .candidates
            .first()
            .context("Gemini response has no candidate responses")?
            .content
            .parts
            .last()
            .context("Failed to extract last llm response")?
            .text
            .clone();
        Ok(text)
    }
}

impl GeminiProvider {
    pub fn new(
        config: &GeminiProviderOpts,
        api_key_manager: &APIKeyManager,
        cli_handler: Option<&CliHandler>,
    ) -> Self {
        Self {
            provider: OnlineProvider::new(
                GeminiProvider::provider_str(),
                &config.online_opts,
                api_key_manager,
                cli_handler,
            ),
            http_client: reqwest::Client::builder()
                .build()
                .expect("Failed to build http client."),
        }
    }
}

mod gemini_api_response {
    use super::*;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GeminiApiResponse {
        pub candidates: Vec<Candidate>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Candidate {
        pub content: Content,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Content {
        pub parts: Vec<Part>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Part {
        pub text: String,
    }
}

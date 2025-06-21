use gemini_api_response::GeminiApiResponse;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{APIKeyManager, cli_handler::CliHandler, configuration::Configuration};

use anyhow::Context;

use super::{
    ChatData, ChatHistoryItem, ChatRole, FileUploadData, GEMINI_PROVIDER, LLMTools, OnlineProvider,
    OnlineProviderImpl, ProviderImpl,
};

#[derive(Debug, Clone)]
pub struct GeminiProvider {
    provider: OnlineProvider,
    http_client: reqwest::Client,
    memory: Vec<ChatHistoryItem>,
    system_prompt: Option<String>,

    gemini_tools: LLMTools,
}

impl ProviderImpl for GeminiProvider {
    fn provider_str() -> &'static str {
        GEMINI_PROVIDER
    }

    fn merge_tools(&mut self, tools: LLMTools) {
        self.gemini_tools.merge(&tools);
    }

    fn update_memory(&mut self, prompt: String, response: String) -> anyhow::Result<()> {
        self.memory.extend(vec![
            ChatData {
                role: ChatRole::User,
                text: prompt,
            }
            .into(),
            ChatData {
                role: ChatRole::Model,
                text: response,
            }
            .into(),
        ]);

        Ok(())
    }

    /// Used when providing model context.
    fn add_chat_to_context(&mut self, chat: ChatHistoryItem) -> anyhow::Result<Option<usize>> {
        match chat {
            ChatHistoryItem::Chat(ChatData {
                role: ChatRole::System,
                text,
            }) => {
                self.system_prompt = Some(text);
                Ok(None)
            }
            _ => {
                self.memory.push(chat);
                Ok(Some(self.memory.len() - 1))
            }
        }
    }

    fn append_chat_in_context(&mut self, index: usize, text: &str) -> anyhow::Result<()> {
        let existing = self
            .memory
            .get_mut(index)
            .context(format!("Failed to get chat at index {}", index))?;
        match existing {
            ChatHistoryItem::FileUpload(file) => {}
            ChatHistoryItem::Chat(chat) => chat.text.push_str(text),
        }
        Ok(())
    }

    fn clear_memory(&mut self) -> anyhow::Result<()> {
        Ok(self.memory.clear())
    }

    fn get_history(&self) -> &Vec<ChatHistoryItem> {
        &self.memory
    }
}

impl OnlineProviderImpl for GeminiProvider {
    type ProviderApiResponse = GeminiApiResponse;
    type ProviderApiStreamResponse = GeminiApiResponse;

    fn build_chat_url(&self) -> anyhow::Result<Url> {
        let mut url = reqwest::Url::parse(&self.provider.url)
            .context("Failed to parse provider url")?
            .join("v1beta/models/")
            .context("Failed to build gemini url.")?
            .join(&self.provider.model)
            .context("Failed to build gemini model url")?;

        url = Url::parse(&(url.to_string() + ":generateContent"))
            .context("Failed to append chat generation type")?;
        url.set_query(Some(format!("key={}", &self.provider.api_key).as_str()));
        Ok(url)
    }

    fn build_chat_stream_url(&self) -> anyhow::Result<reqwest::Url> {
        let mut url = reqwest::Url::parse(&self.provider.url)
            .context("Failed to parse provider url")?
            .join("v1beta/models/")
            .context("Failed to build gemini url.")?
            .join(&self.provider.model)
            .context("Failed to build gemini model url")?;

        url = Url::parse(&(url.to_string() + ":streamGenerateContent"))
            .context("Failed to append chat generation type")?;
        url.set_query(Some(
            format!("key={}&alt=sse", &self.provider.api_key).as_str(),
        ));
        Ok(url)
    }

    fn build_chat_body(&self, prompt: impl Into<String>) -> serde_json::Value {
        let system_prompt = if let Some(instructions) = &self.system_prompt {
            json!({
                "parts": [
                    {
                        "text": instructions
                    }
                ]
            })
        } else {
            json!(null)
        };
        let mut temp_chat_hist = self
            .memory
            .iter()
            .filter(|item| match item {
                ChatHistoryItem::Chat(ChatData {
                    role: ChatRole::System,
                    text: _,
                }) => false,
                _ => true,
            })
            .map(|chat| Self::serialise_chat(chat))
            .collect::<Vec<_>>();

        let new_chat = ChatData {
            role: ChatRole::User,
            text: prompt.into(),
        }
        .into();
        temp_chat_hist.push(Self::serialise_chat(&new_chat));

        json!({
            "system_instruction": system_prompt,
            "contents": temp_chat_hist,
            "tools": self.build_tools()
        })
    }

    fn get_http_client(&self) -> &reqwest::Client {
        &self.http_client
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

    fn decode_llm_stream_response(
        &self,
        response: Self::ProviderApiStreamResponse,
    ) -> anyhow::Result<String> {
        self.decode_llm_response(response)
    }
}

impl GeminiProvider {
    pub fn new(
        config: &Configuration,
        api_key_manager: &APIKeyManager,
        cli_handler: Option<&CliHandler>,
    ) -> Self {
        Self {
            provider: OnlineProvider::new(
                GeminiProvider::provider_str(),
                &config.provider_opts.gemini.online_opts,
                api_key_manager,
                cli_handler,
            ),
            http_client: reqwest::Client::builder()
                .build()
                .expect("Failed to build http client."),
            memory: Vec::new(),
            gemini_tools: LLMTools::new(config),
            system_prompt: None,
        }
    }
}

impl GeminiProvider {
    fn serialise_chat(chat: &ChatHistoryItem) -> serde_json::Value {
        match chat {
            ChatHistoryItem::Chat(chat) => {
                let role = match chat.role {
                    ChatRole::Model => "model",
                    ChatRole::System => "system_instruction",
                    ChatRole::User => "user",
                };

                json!({
                    "role": role,
                    "parts": [
                        {
                            "text": chat.text
                        }
                    ]
                })
            }
            ChatHistoryItem::FileUpload(file) => {
                let role = ChatRole::User;
                json!({
                    "role": role,
                    "parts": [
                        {
                            "text": format!("## <{}> Contents below ##\n{}",file.relative_filepath, file.text)
                        }
                    ]
                })
            }
        }
    }

    fn build_tools(&self) -> serde_json::Value {
        let mut enabled_tools = Vec::new();
        if self.gemini_tools.search {
            enabled_tools.push(json!({ "google_search": {}}));
        }
        json!(enabled_tools)
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

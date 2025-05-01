use crate::{cli_handler::CliHandler, configuration::GeminiProviderOpts};

use super::{APIKeyManager, GEMINI_PROVIDER, OnlineProvider, ProviderImpl};

pub struct GeminiProvider {
    provider: OnlineProvider,
    http_client: reqwest::Client,
}

impl ProviderImpl for GeminiProvider {
    fn complete_chat(&self, prompt: String) -> String {
        self.http_client.post(self.provider.url)
    }

    fn provider_str() -> &'static str {
        GEMINI_PROVIDER
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

    fn get_chat_url(&self) -> String {}
}

use super::{OnlineProvider, ProviderImpl};

pub struct GeminiProvider {
    provider: OnlineProvider,
    http_client: reqwest::Client,
}

impl ProviderImpl for GeminiProvider {
    fn complete_chat(&self, prompt: String) -> String {}

    fn new(config: &crate::configuration::Configuration) -> Self {
        Self {
            provider: OnlineProvider {
                api_key: (),
                url: (),
            },
            http_client: reqwest::Client::builder()
                .build()
                .expect("Failed to build http client."),
        }
    }
}

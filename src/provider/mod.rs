mod gemini;

use gemini::GeminiProvider;

use crate::configuration::Configuration;

struct OnlineProvider {
    api_key: String,
    url: String,
}

trait ProviderImpl {
    fn complete_chat(&self, prompt: String) -> String;

    fn new(config: &Configuration) -> Self;
}

pub enum Provider {
    Gemini(GeminiProvider),
}

impl Provider {
    pub fn complete_chat(&self, prompt: String) -> String {
        match self {
            Self::Gemini(prov) => prov.complete_chat(prompt),
        }
    }
}

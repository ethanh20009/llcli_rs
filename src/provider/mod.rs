mod error;
mod gemini;

use gemini::GeminiProvider;

use crate::configuration::Configuration;
use crate::{cli_handler::CliHandler, configuration::OnlineProviderOpts};
use error::{Error, Result};

struct OnlineProvider {
    api_key: String,
    url: String,
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
            (Err(Error::NoApiKey), Some(cli_handler)) => cli_handler.get_api_key(),
            (Err(Error::NoApiKey), None) => panic!("No api key found. Please add an api key."),
            (Err(Error::KeyFetchError(err)), _) => {
                panic!("Error fetching key from keychain. {:?}", err)
            }
        };
        Self {
            url: config.url.clone(),
            api_key,
        }
    }
}

trait ProviderImpl {
    fn complete_chat(&self, prompt: String) -> String;

    fn provider_str() -> &'static str;
}

#[derive(derive_more::From)]
pub enum Provider {
    Gemini(GeminiProvider),
}

const GEMINI_PROVIDER: &'static str = "gemini";

impl Provider {
    pub fn complete_chat(&self, prompt: String) -> String {
        match self {
            Self::Gemini(prov) => prov.complete_chat(prompt),
        }
    }

    pub fn new(
        &self,
        config: &Configuration,
        api_key_manager: &APIKeyManager,
        cli_handler: Option<&CliHandler>,
        provider: &str,
    ) -> Provider {
        match provider {
            GEMINI_PROVIDER => {
                GeminiProvider::new(&config.provider_opts.gemini, api_key_manager, cli_handler)
                    .into()
            }
            _ => panic!(
                "invalid provider string reference. Recieved: {:?}",
                provider
            ),
        }
    }
}

struct APIKeyManager {
    user_name: String,
}

const SERVICE_NAME: &'static str = "llmcli";

impl APIKeyManager {
    fn new() -> Self {
        Self {
            user_name: whoami::username(),
        }
    }
    fn fetch_api_key(&self, provider: &str) -> Result<String> {
        let entry = keyring::Entry::new_with_target(provider, SERVICE_NAME, &self.user_name)
            .expect("Failed to create keyring entry with target.");
        match entry.get_password() {
            Ok(password) => Ok(password),
            Err(err) => match err {
                keyring::Error::NoEntry => Err(Error::NoApiKey),
                _ => Err(err.into()),
            },
        }
    }
}

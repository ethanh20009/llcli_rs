use tracing::instrument;

use super::{Error, Result};

#[derive(Debug)]
pub struct APIKeyManager {
    user_name: String,
}

const SERVICE_NAME: &'static str = "llmcli_rs";

impl APIKeyManager {
    #[instrument(ret)]
    pub fn new() -> Self {
        Self {
            user_name: whoami::username(),
        }
    }
    pub fn fetch_api_key(&self, provider: &str) -> Result<String> {
        let entry = keyring::Entry::new_with_target(provider, SERVICE_NAME, &self.user_name)
            .expect("Failed to create keyring entry with target.");
        match entry.get_password() {
            Ok(password) => {
                entry.set_password(password.as_str())?;
                Ok(password)
            }
            Err(err) => match err {
                keyring::Error::NoEntry => Err(Error::NoApiKey),
                _ => Err(err.into()),
            },
        }
    }

    pub fn set_api_key(&self, provider: &str, key: &str) -> Result<String> {
        let entry = keyring::Entry::new_with_target(provider, SERVICE_NAME, &self.user_name)
            .expect("Failed to create keyring entry with target.");
        match entry.set_password(key) {
            Ok(_) => Ok(key.to_string()),
            Err(err) => Err(err.into()),
        }
    }
}

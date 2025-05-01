use config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    provider: String,
    provider_opts: ProviderOpts,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderOpts {
    Gemini(GeminiProviderOpts),
}

#[derive(Serialize, Deserialize)]
pub struct OnlineProviderOpts {
    url: String,
    api_key: Option<String>,
}

pub struct ConfigManager {
    config: Configuration,
}

impl ConfigManager {
    pub fn new() -> ConfigManager {
        let config_dir = dirs::config_dir().expect("Failed to retrieve config dir path.");
        let settings_path = config_dir.join("llcli.toml");
        let settings = config::Config::builder()
            .add_source(
                config::File::with_name(
                    settings_path
                        .to_str()
                        .expect("Failed to unwrap settings file path."),
                )
                .required(false),
            )
            .build()
            .expect("Failed to build config")
            .try_deserialize::<Configuration>()
            .expect("Failed to deserialise config file");
        Self { config: settings }
    }
}

// Provider Opts

#[derive(Serialize, Deserialize)]
pub struct GeminiProviderOpts {
    #[serde(flatten)]
    online_opts: OnlineProviderOpts,
}

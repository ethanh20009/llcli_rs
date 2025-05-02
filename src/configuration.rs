use config;
use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG: &str = include_str!("default_config.toml");

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub provider: String,
    pub provider_opts: ProviderOpts,
    pub tools: Option<ToolOptions>,
}

#[derive(Serialize, Deserialize)]
pub struct ProviderOpts {
    pub gemini: GeminiProviderOpts,
}

#[derive(Serialize, Deserialize)]
pub struct OnlineProviderOpts {
    pub url: String,
    pub model: String,
}

#[derive(Serialize, Deserialize)]
pub struct ToolOptions {
    pub search_default: Option<bool>,
}

pub struct ConfigManager {
    pub config: Configuration,
}

impl ConfigManager {
    pub fn new() -> ConfigManager {
        let config_dir = dirs::config_dir().expect("Failed to retrieve config dir path.");
        let settings_path = config_dir.join("llcli.toml");
        let settings = config::Config::builder()
            .add_source(config::File::from_str(
                DEFAULT_CONFIG,
                config::FileFormat::Toml,
            ))
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
    pub online_opts: OnlineProviderOpts,
}

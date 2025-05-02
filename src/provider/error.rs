pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Fetching api key for provider. {0}")]
    KeyFetchError(#[from] keyring::Error),
    #[error("No api key stored")]
    NoApiKey,
}

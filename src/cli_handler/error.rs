use inquire::InquireError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("User interrupted.")]
    Interrupted,
    #[error("Error retrieving input. {0}")]
    InquiryError(#[from] InquireError),
    #[error("Command: {0} not an option.")]
    CommandNotOption(String),
    #[error("File Handler error. {0}")]
    FileHandlerError(#[from] anyhow::Error),
    #[error("LLM Stream error. {0}")]
    LLMError(anyhow::Error),
}

pub fn map_inquire_error(err: InquireError) -> Error {
    match err {
        InquireError::OperationInterrupted | InquireError::OperationCanceled => Error::Interrupted,
        _ => Error::from(err),
    }
}

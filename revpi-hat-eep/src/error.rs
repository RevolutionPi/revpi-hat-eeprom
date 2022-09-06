

#[derive(thiserror::Error, Debug)]
pub enum RevPiError {
    #[error("JSON parse error")]
    JsonError(#[from] serde_json::Error),
    #[error("Config validation error")]
    Error(String),
    #[error("Validation error")]
    ValidationError(String),
    #[error("unknown error")]
    Unknown,
}

impl From<String> for RevPiError {
    fn from(str: String) -> Self {
        RevPiError::Error(str)
    }
}
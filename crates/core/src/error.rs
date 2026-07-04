use thiserror::Error;

pub type OmegaResult<T> = Result<T, OmegaError>;

#[derive(Debug, Error)]
pub enum OmegaError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("provider error: {0}")]
    Provider(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("matcher error: {0}")]
    Matcher(String),

    #[error("XMLTV error: {0}")]
    Xmltv(String),

    #[error("playlist error: {0}")]
    Playlist(String),

    #[error("publish error: {0}")]
    Publish(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("task timeout")]
    Timeout,

    #[error("unknown error: {0}")]
    Unknown(String),
}

impl OmegaError {
    pub fn provider(message: impl Into<String>) -> Self {
        Self::Provider(message.into())
    }

    pub fn matcher(message: impl Into<String>) -> Self {
        Self::Matcher(message.into())
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::Database(message.into())
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON deserialization error: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("Too many retries")]
    TooManyRetry(Vec<Error>),
}

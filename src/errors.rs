//! # Errors
//!
//! Errors that can occur when interacting with the LastFM Client.
use crate::error_response::ErrorResponse;
use thiserror::Error;

/// Errors that can occur when interacting with the LastFM Client.
#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON deserialization error: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("Unretryable error from LastFM: {0}")]
    UnretriableLastFm(#[from] ErrorResponse),
    #[error("Too many retries")]
    TooManyRetry(Vec<Error>),
}

use reqwest::Client;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FerriskeyApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
}

#[derive(Clone)]
pub struct FerriskeyApi {
    _client: Client,
    _base_url: String,
}

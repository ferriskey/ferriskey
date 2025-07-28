use axum::extract::FromRequestParts;
use base64::{Engine, engine::general_purpose};
use serde::Deserialize;

use crate::domain::jwt::entities::jwt_error::JwtError;

#[derive(Debug, Clone, Deserialize)]
pub struct DecodedToken {
    pub sub: String,
    pub preferred_username: String,
}

#[derive(Debug, Clone)]
pub struct ResultToken {
    pub token: String,
    pub decoded_token: DecodedToken,
}

/// Wrapper type for optional JWT token extraction
/// This allows us to implement FromRequestParts while respectign Rust's orphan rules.
#[derive(Debug, Clone)]
pub struct OptionalToken(pub Option<ResultToken>);

impl<S> FromRequestParts<S> for OptionalToken
where
    S: Send + Sync,
{
    type Rejection = axum::response::Response;

    /// Extracts an optional decoded JWT token from the Authorization header.
    ///
    /// This implementation attempts to parse a Bearer token from the Authorization
    /// header and decode its payload. If any step fails (missing header, invalid
    /// format, decode errors), it returns Ok(None) rather than an error.
    ///
    /// # Returns
    /// - `Ok(OptionalToken(Some(DecodedToken)))` if a valid token is found and decoded
    /// - `Ok(OptionalToken(None))` if no token is present or token is invalid
    /// - `Err(Response)` only in exceptional cases (currently never)
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|hv| hv.to_str().ok())
            .and_then(|auth| auth.strip_prefix("Bearer "))
            .filter(|token| !token.is_empty());

        let Some(token) = token else {
            return Ok(OptionalToken(None));
        };

        match parse_jwt_payload(token) {
            Ok(decoded_token) => Ok(OptionalToken(Some(ResultToken {
                token: token.to_string(),
                decoded_token,
            }))),
            Err(_) => Ok(OptionalToken(None)),
        }
    }
}

fn parse_jwt_payload(token: &str) -> Result<DecodedToken, JwtError> {
    let parts: Vec<&str> = token.split('.').collect();

    if parts.len() != 3 {
        return Err(JwtError::InvalidToken);
    }

    let payload = parts[1];

    let decoded_bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|e| JwtError::ParsingError(e.to_string()))?;

    let payload_str =
        std::str::from_utf8(&decoded_bytes).map_err(|e| JwtError::ParsingError(e.to_string()))?;

    serde_json::from_str::<DecodedToken>(payload_str)
        .map_err(|e| JwtError::ParsingError(e.to_string()))
}

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorBody};
use ferriskey_api_core::decoded_token::OptionalToken;
use uuid::Uuid;

pub struct CallerSession(pub Uuid);

impl<S> FromRequestParts<S> for CallerSession
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = OptionalToken::from_request_parts(parts, state)
            .await
            .map_err(|_| unauthorized())?;

        token
            .0
            .and_then(|token| token.claims.sid)
            .map(CallerSession)
            .ok_or_else(unauthorized)
    }
}

fn unauthorized() -> ApiError {
    ApiError::Unauthorized(ApiErrorBody::new(
        "This operation requires a session-bound access token",
        "session_bound_token_required",
    ))
}

use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
    caller_session::CallerSession,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RevokeOtherSessionsResponse {
    pub revoked: u64,
}

#[utoipa::path(
    post,
    path = "/me/sessions/revoke-others",
    tag = "account",
    summary = "Revoke every session of the caller except the current one",
    description = "Revokes every live session of the caller other than the one the request was made with, each through the same path as an administrator revoking it: tokens stop working and the clients that took part get a back-channel logout. The current session is kept. The user id is taken from the access token, never from the URL, and the token must be bound to a session.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Other sessions revoked", body = RevokeOtherSessionsResponse),
        (status = 401, description = "The token is not bound to a session", body = ApiErrorResponse),
        (status = 403, description = "The caller is not a regular user", body = ApiErrorResponse),
        (status = 404, description = "No such realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn revoke_other_own_sessions(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CallerSession(current_session_id): CallerSession,
) -> Result<Response<RevokeOtherSessionsResponse>, ApiError> {
    let revoked = state
        .service
        .revoke_other_own_sessions(identity, realm_name, current_session_id)
        .await?;

    Ok(Response::OK(RevokeOtherSessionsResponse { revoked }))
}

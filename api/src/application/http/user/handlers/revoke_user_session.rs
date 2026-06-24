use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::api_error::{ApiError, ApiErrorResponse},
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RevokeSessionResponse {}

#[utoipa::path(
    delete,
    path = "/{user_id}/sessions/{session_id}",
    tag = "user",
    summary = "Revoke a specific user session",
    description = "Revokes a single session for the given user. The session's tokens are immediately invalidated. Other sessions are unaffected. Requires ManageUsers permission, unless the caller is the user themselves.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
        ("session_id" = Uuid, Path, description = "Session ID to revoke"),
    ),
    responses(
        (status = 204, description = "Session revoked successfully"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Session not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn revoke_user_session(
    Path((realm_name, user_id, session_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .service
        .revoke_user_session(identity, realm_name, user_id, session_id)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ferriskey_api_core::{
    api_entities::api_error::{ApiError, ApiErrorResponse},
    app_state::AppState,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/me/sessions/{session_id}",
    tag = "account",
    summary = "Revoke one of the caller's own sessions",
    description = "Revokes a single session of the caller, through the same path as an administrator revoking it: its refresh and access tokens stop working and the clients that took part in it get a back-channel logout. Revoking the session the request was made with signs the caller out. A session that does not exist or belongs to someone else is a 404. The user id is taken from the access token, never from the URL.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("session_id" = Uuid, Path, description = "Session ID to revoke"),
    ),
    responses(
        (status = 204, description = "Session revoked"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "The caller is not a regular user", body = ApiErrorResponse),
        (status = 404, description = "Realm or session not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn revoke_own_session(
    Path((realm_name, session_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .service
        .revoke_own_session(identity, realm_name, session_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

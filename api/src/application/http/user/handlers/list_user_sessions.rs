use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserSessionDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListUserSessionsResponse {
    pub data: Vec<UserSessionDto>,
}

#[utoipa::path(
    get,
    path = "/{user_id}/sessions",
    tag = "user",
    summary = "List active sessions for a user",
    description = "Returns all active sessions for the given user in the realm. Requires ManageUsers or ViewUsers permission, unless the caller is the user themselves.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "Sessions retrieved successfully", body = ListUserSessionsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn list_user_sessions(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ListUserSessionsResponse>, ApiError> {
    let sessions = state
        .service
        .list_user_sessions(identity, realm_name, user_id)
        .await
        .map_err(ApiError::from)?;

    let data = sessions
        .into_iter()
        .map(|s| UserSessionDto {
            id: s.id,
            user_id: s.user_id,
            realm_id: s.realm_id,
            user_agent: s.user_agent,
            ip_address: s.ip_address,
            created_at: s.created_at,
            expires_at: s.expires_at,
            last_seen_at: s.last_seen_at,
        })
        .collect();

    Ok(Response::OK(ListUserSessionsResponse { data }))
}

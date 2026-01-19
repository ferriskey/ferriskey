use axum::extract::{Path, State};
use ferriskey_core::domain::abyss::federation::{entities::SyncMode, ports::FederationService};
use uuid::Uuid;

use crate::application::http::{
    abyss::federation::dto::SyncUsersResponse,
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};

#[utoipa::path(
    post,
    path = "/federation/providers/{id}/sync-users",
    summary = "Sync Users from Federation Provider",
    description = "Triggers synchronization of users from the external federation provider (LDAP, Kerberos, etc.) to the local IAM database. This performs a comprehensive diff and reconciliation.",
    responses(
        (status = 200, description = "Sync completed successfully", body = SyncUsersResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Provider not found"),
        (status = 500, description = "Sync failed"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("id" = String, Path, description = "Provider ID")
    ),
    tag = "federation"
)]
pub async fn sync_users(
    Path((_, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
) -> Result<Response<SyncUsersResponse>, ApiError> {
    // Default to Import mode for safety (Force would disable missing users)
    let result = state
        .service
        .sync_federation_users(id, SyncMode::Import)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(result.into()))
}

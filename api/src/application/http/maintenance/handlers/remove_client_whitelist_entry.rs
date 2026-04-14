use crate::application::http::server::api_entities::api_error::{ApiError, ApiErrorResponse};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::maintenance::ports::MaintenanceService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RemoveClientWhitelistEntryResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/{client_id}/maintenance/whitelist/{entry_id}",
    tag = "maintenance",
    summary = "Remove entry from client maintenance whitelist",
    description = "Removes a user or role from the client's maintenance whitelist.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("entry_id" = Uuid, Path, description = "Whitelist entry ID"),
    ),
    responses(
        (status = 200, description = "Entry removed", body = RemoveClientWhitelistEntryResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Entry not found", body = ApiErrorResponse),
    ),
)]
pub async fn remove_client_whitelist_entry(
    Path((realm_name, client_id, entry_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<RemoveClientWhitelistEntryResponse>, ApiError> {
    state
        .service
        .remove_client_whitelist_entry(identity, realm_name, client_id, entry_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(RemoveClientWhitelistEntryResponse {
        message: "Entry removed".to_string(),
    }))
}

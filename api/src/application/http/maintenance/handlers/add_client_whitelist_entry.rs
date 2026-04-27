use crate::application::http::maintenance::validators::AddWhitelistEntryValidator;
use crate::application::http::server::api_entities::api_error::{
    ApiError, ApiErrorResponse, ValidateJson,
};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::maintenance::entities::MaintenanceWhitelistEntry;
use ferriskey_core::domain::maintenance::ports::MaintenanceService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct AddClientWhitelistEntryResponse {
    pub data: MaintenanceWhitelistEntry,
}

#[utoipa::path(
    post,
    path = "/{client_id}/maintenance/whitelist",
    tag = "maintenance",
    summary = "Add entry to client maintenance whitelist",
    description = "Adds a user or role to the client's maintenance whitelist. Exactly one of user_id or role_id must be provided.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    request_body = AddWhitelistEntryValidator,
    responses(
        (status = 201, description = "Entry added", body = AddClientWhitelistEntryResponse),
        (status = 400, description = "Invalid request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
    ),
)]
pub async fn add_client_whitelist_entry(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<AddWhitelistEntryValidator>,
) -> Result<Response<AddClientWhitelistEntryResponse>, ApiError> {
    let entry = match (payload.user_id, payload.role_id) {
        (Some(user_id), None) => state
            .service
            .add_client_whitelist_user(identity, realm_name, client_id, user_id)
            .await
            .map_err(ApiError::from)?,
        (None, Some(role_id)) => state
            .service
            .add_client_whitelist_role(identity, realm_name, client_id, role_id)
            .await
            .map_err(ApiError::from)?,
        _ => {
            return Err(ApiError::BadRequest(
                "Exactly one of user_id or role_id must be provided".into(),
            ));
        }
    };

    Ok(Response::Created(AddClientWhitelistEntryResponse {
        data: entry,
    }))
}

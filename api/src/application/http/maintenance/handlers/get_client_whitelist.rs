use crate::application::http::server::api_entities::api_error::{ApiError, ApiErrorResponse};
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
pub struct GetClientWhitelistResponse {
    pub data: Vec<MaintenanceWhitelistEntry>,
}

#[utoipa::path(
    get,
    path = "/{client_id}/maintenance/whitelist",
    tag = "maintenance",
    summary = "Get client maintenance whitelist",
    description = "Returns the list of users and roles whitelisted for this client during maintenance mode.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    responses(
        (status = 200, description = "Whitelist retrieved", body = GetClientWhitelistResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
    ),
)]
pub async fn get_client_whitelist(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetClientWhitelistResponse>, ApiError> {
    let entries = state
        .service
        .get_client_whitelist(identity, realm_name, client_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetClientWhitelistResponse { data: entries }))
}

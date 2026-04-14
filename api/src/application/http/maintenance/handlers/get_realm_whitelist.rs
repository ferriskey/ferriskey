use crate::application::http::server::api_entities::api_error::{ApiError, ApiErrorResponse};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::maintenance::entities::RealmMaintenanceWhitelistEntry;
use ferriskey_core::domain::maintenance::ports::MaintenanceService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetRealmWhitelistResponse {
    pub data: Vec<RealmMaintenanceWhitelistEntry>,
}

#[utoipa::path(
    get,
    path = "/settings/maintenance/whitelist",
    tag = "maintenance",
    summary = "Get realm maintenance whitelist",
    description = "Returns the list of users and roles automatically whitelisted on all clients under maintenance in this realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Whitelist retrieved", body = GetRealmWhitelistResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
    ),
)]
pub async fn get_realm_whitelist(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetRealmWhitelistResponse>, ApiError> {
    let entries = state
        .service
        .get_realm_whitelist(identity, realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetRealmWhitelistResponse { data: entries }))
}

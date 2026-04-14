use crate::application::http::maintenance::validators::ToggleMaintenanceValidator;
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
use ferriskey_core::domain::maintenance::ports::MaintenanceService;
use ferriskey_core::domain::maintenance::value_objects::ToggleMaintenanceRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToggleMaintenanceResponse {
    pub message: String,
}

#[utoipa::path(
    put,
    path = "/{client_id}/maintenance",
    tag = "maintenance",
    summary = "Toggle client maintenance mode",
    description = "Enables or disables maintenance mode for a client. When enabled, only whitelisted users and roles can authenticate.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    request_body = ToggleMaintenanceValidator,
    responses(
        (status = 200, description = "Maintenance mode toggled successfully", body = ToggleMaintenanceResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
    ),
)]
pub async fn toggle_maintenance(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ToggleMaintenanceValidator>,
) -> Result<Response<ToggleMaintenanceResponse>, ApiError> {
    state
        .service
        .toggle_maintenance(
            identity,
            realm_name,
            client_id,
            ToggleMaintenanceRequest {
                enabled: payload.enabled,
                reason: payload.reason,
                session_strategy: payload.session_strategy,
            },
        )
        .await
        .map_err(ApiError::from)?;

    let status = if payload.enabled {
        "enabled"
    } else {
        "disabled"
    };

    Ok(Response::OK(ToggleMaintenanceResponse {
        message: format!("Maintenance mode {status}"),
    }))
}

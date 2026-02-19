use axum::{
    Extension, Json,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    abyss::federation::{
        entities::{FederationType, SyncMode},
        ports::FederationService,
        value_objects::UpdateProviderRequest as CoreUpdateProviderRequest,
    },
    authentication::value_objects::Identity,
};
use serde::Serialize;
use std::str::FromStr;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::{
    abyss::federation::dto::{ProviderResponse, UpdateProviderRequest},
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateProviderResponse {
    pub data: ProviderResponse,
}

#[utoipa::path(
    put,
    path = "/federation/providers/{id}",
    summary = "Update a federation provider",
    request_body = UpdateProviderRequest,
    responses(
        (status = 200, description = "Provider updated", body = UpdateProviderResponse),
        (status = 400, description = "Invalid input", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 404, description = "Realm or Provider not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("id" = String, Path, description = "Provider ID")
    ),
    tag = "federation"
)]
pub async fn update_provider(
    Path((realm_name, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateProviderRequest>,
) -> Result<Response<UpdateProviderResponse>, ApiError> {
    let provider_type = payload.provider_type.map(|pt| match pt.as_str() {
        "Ldap" => FederationType::Ldap,
        "Kerberos" => FederationType::Kerberos,
        "ActiveDirectory" => FederationType::ActiveDirectory,
        s => FederationType::Custom(s.to_string()),
    });

    let sync_settings = if payload.sync_enabled.is_some()
        || payload.sync_mode.is_some()
        || payload.sync_interval_minutes.is_some()
    {
        let enabled = payload.sync_enabled.unwrap_or(false);
        let mode_str = payload.sync_mode.clone().unwrap_or("LinkOnly".to_string()); // Clone because we consume payload

        let _ = SyncMode::from_str(&mode_str)
            .map_err(|_| ApiError::BadRequest(format!("Invalid sync mode: {}", mode_str)))?;

        let interval = payload.sync_interval_minutes; // Option<i32>

        Some(serde_json::json!({
            "enabled": enabled,
            "mode": mode_str,
            "interval_minutes": interval
        }))
    } else {
        None
    };

    let core_request = CoreUpdateProviderRequest {
        name: payload.name,
        provider_type,
        enabled: payload.enabled,
        priority: payload.priority,
        config: payload.config,
        sync_settings,
    };

    // 3. Update
    let provider = state
        .service
        .update_federation_provider(identity, realm_name, id, core_request)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateProviderResponse {
        data: provider.into(),
    }))
}

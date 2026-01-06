use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use ferriskey_core::domain::{
    abyss::federation::{
        entities::{FederationType, SyncMode},
        ports::FederationService,
        value_objects::UpdateProviderRequest as CoreUpdateProviderRequest,
    },
    authentication::value_objects::Identity,
    realm::ports::{GetRealmInput, RealmService},
};
use std::str::FromStr;
use uuid::Uuid;

use crate::application::http::{
    abyss::federation::dto::{ProviderResponse, UpdateProviderRequest},
    server::api_entities::api_error::ApiError,
    server::app_state::AppState,
};

#[utoipa::path(
    put,
    path = "/federation/providers/{id}",
    request_body = UpdateProviderRequest,
    responses(
        (status = 200, description = "Provider updated", body = ProviderResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Realm or Provider not found"),
    ),
    params(
        ("realm" = String, Path, description = "Realm name"),
        ("id" = String, Path, description = "Provider ID")
    ),
    tag = "Federation"
)]
pub async fn update_provider(
    Path((realm_name, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateProviderRequest>,
) -> Result<(StatusCode, Json<ProviderResponse>), ApiError> {
    // 1. Verify realm access
    let realm = state
        .service
        .get_realm_by_name(identity.clone(), GetRealmInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    // 2. Verify existence and ownership
    let existing = state
        .service
        .get_federation_provider(id)
        .await
        .map_err(ApiError::from)?;

    if existing.realm_id != Into::<Uuid>::into(realm.id) {
        return Err(ApiError::NotFound(
            "Provider not found in this realm".to_string(),
        ));
    }

    // 3. Map Request
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
        let existing_sync = &existing.sync_settings;

        let enabled = payload.sync_enabled.unwrap_or(
            existing_sync
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        );

        let mode_str = if let Some(m) = payload.sync_mode {
            m
        } else {
            existing_sync
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("LinkOnly")
                .to_string()
        };

        let _ = SyncMode::from_str(&mode_str)
            .map_err(|_| ApiError::BadRequest(format!("Invalid sync mode: {}", mode_str)))?;

        let interval = if payload.sync_interval_minutes.is_some() {
            payload.sync_interval_minutes
        } else {
            existing_sync
                .get("interval_minutes")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
        };

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

    // 4. Update
    let provider = state
        .service
        .update_federation_provider(id, core_request)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(ProviderResponse::from(provider))))
}

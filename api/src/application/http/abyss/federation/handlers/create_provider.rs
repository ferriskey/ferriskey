use axum::{
    Extension, Json,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    abyss::federation::{
        entities::{FederationType, SyncMode},
        ports::FederationService,
        value_objects::CreateProviderRequest as CoreCreateProviderRequest,
    },
    authentication::value_objects::Identity,
};
use std::str::FromStr;
use uuid::Uuid;

use crate::application::http::{
    abyss::federation::dto::{CreateProviderRequest, ProviderResponse},
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};

#[utoipa::path(
    post,
    path = "/federation/providers",
    summary = "Create Federation Provider",
    request_body = CreateProviderRequest,
    responses(
        (status = 201, description = "Provider created", body = ProviderResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Realm not found"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name")
    ),
    tag = "federation"
)]
pub async fn create_provider(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateProviderRequest>,
) -> Result<Response<ProviderResponse>, ApiError> {
    // 1. Parse Enums
    let provider_type = match payload.provider_type.as_str() {
        "Ldap" => FederationType::Ldap,
        "Kerberos" => FederationType::Kerberos,
        "ActiveDirectory" => FederationType::ActiveDirectory,
        s => FederationType::Custom(s.to_string()),
    };

    let _sync_mode = SyncMode::from_str(&payload.sync_mode)
        .map_err(|_| ApiError::BadRequest(format!("Invalid sync mode: {}", payload.sync_mode)))?;

    // 2. Construct Core Request
    let sync_settings = serde_json::json!({
        "enabled": payload.sync_enabled,
        "mode": payload.sync_mode,
        "interval_minutes": payload.sync_interval_minutes
    });

    let core_request = CoreCreateProviderRequest {
        realm_id: Uuid::default(), // Service will set this
        name: payload.name,
        provider_type,
        enabled: payload.enabled,
        priority: payload.priority,
        config: payload.config,
        sync_settings,
    };

    // 3. Call Service
    let provider = state
        .service
        .create_federation_provider(identity, realm_name, core_request)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Created(provider.into()))
}

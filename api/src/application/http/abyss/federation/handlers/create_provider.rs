use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use ferriskey_core::domain::{
    abyss::federation::{
        entities::{FederationType, SyncMode},
        ports::FederationService,
        value_objects::CreateProviderRequest as CoreCreateProviderRequest,
    },
    authentication::value_objects::Identity,
    realm::ports::{GetRealmInput, RealmService},
};
use std::str::FromStr;

use crate::application::http::{
    abyss::federation::dto::{CreateProviderRequest, ProviderResponse},
    server::api_entities::api_error::ApiError,
    server::app_state::AppState,
};

#[utoipa::path(
    post,
    path = "/federation/providers",
    request_body = CreateProviderRequest,
    responses(
        (status = 201, description = "Provider created", body = ProviderResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Realm not found"),
    ),
    params(
        ("realm" = String, Path, description = "Realm name")
    ),
    tag = "Federation"
)]
pub async fn create_provider(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateProviderRequest>,
) -> Result<(StatusCode, Json<ProviderResponse>), ApiError> {
    // 1. Get realm to verify it exists and get ID
    let realm = state
        .service
        .get_realm_by_name(
            identity.clone(),
            GetRealmInput {
                realm_name: realm_name.clone(),
            },
        )
        .await
        .map_err(ApiError::from)?;

    // 2. Parse Enums
    let provider_type = match payload.provider_type.as_str() {
        "Ldap" => FederationType::Ldap,
        "Kerberos" => FederationType::Kerberos,
        "ActiveDirectory" => FederationType::ActiveDirectory,
        s => FederationType::Custom(s.to_string()),
    };

    let _sync_mode = SyncMode::from_str(&payload.sync_mode)
        .map_err(|_| ApiError::BadRequest(format!("Invalid sync mode: {}", payload.sync_mode)))?;

    // 3. Construct Core Request
    let sync_settings = serde_json::json!({
        "enabled": payload.sync_enabled,
        "mode": payload.sync_mode,
        "interval_minutes": payload.sync_interval_minutes
    });

    let core_request = CoreCreateProviderRequest {
        realm_id: realm.id.into(),
        name: payload.name,
        provider_type,
        enabled: payload.enabled,
        priority: payload.priority,
        config: payload.config,
        sync_settings,
    };

    // 4. Call Service
    let provider = state
        .service
        .create_federation_provider(core_request)
        .await
        .map_err(ApiError::from)?;

    // 5. Response
    Ok((StatusCode::CREATED, Json(ProviderResponse::from(provider))))
}

use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::{
    authentication::value_objects::Identity, jwt::signing_keys::SigningKey,
    realm::ports::SigningKeyService,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct SigningKeysResponse {
    pub data: Vec<SigningKey>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct SigningKeyResponse {
    pub data: SigningKey,
}

#[utoipa::path(
    get,
    path = "/{realm_name}/keys",
    tag = "realm",
    summary = "List the signing keys of a realm",
    description = "Lists every key the realm has signed with, newest first. The active key signs; passive keys are still published for verification until `retire_at`; retired and revoked keys are kept for audit and never published. No private material is returned.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Signing keys", body = SigningKeysResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_signing_keys(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SigningKeysResponse>, ApiError> {
    state
        .service
        .list_signing_keys(identity, realm_name)
        .await
        .map(|data| Response::OK(SigningKeysResponse { data }))
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/{realm_name}/keys/rotate",
    tag = "realm",
    summary = "Rotate the signing key of a realm",
    description = "Generates a new active key that signs from now on. The previous active key becomes passive: it stays in the JWKS and the SAML descriptor until the longest token lifetime of the realm, client overrides included, has passed, then retires.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 201, description = "The new active key", body = SigningKeyResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn rotate_signing_key(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SigningKeyResponse>, ApiError> {
    state
        .service
        .rotate_signing_key(identity, realm_name)
        .await
        .map(|data| Response::Created(SigningKeyResponse { data }))
        .map_err(ApiError::from)
}

#[utoipa::path(
    post,
    path = "/{realm_name}/keys/{kid}/revoke",
    tag = "realm",
    summary = "Revoke a signing key of a realm",
    description = "Withdraws the key from the JWKS and the SAML descriptor at once: every token it signed stops verifying. Revoking the active key installs a new active key first. Revoking a key twice changes nothing.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("kid" = Uuid, Path, description = "Key id, as published in the JWKS"),
    ),
    responses(
        (status = 200, description = "The revoked key", body = SigningKeyResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "The realm has no such key", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn revoke_signing_key(
    Path((realm_name, kid)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SigningKeyResponse>, ApiError> {
    state
        .service
        .revoke_signing_key(identity, realm_name, kid)
        .await
        .map(|data| Response::OK(SigningKeyResponse { data }))
        .map_err(ApiError::from)
}

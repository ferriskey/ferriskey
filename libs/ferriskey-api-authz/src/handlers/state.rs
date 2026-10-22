use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_authz::tenant::ports::AuthzManagementService;
use ferriskey_core::domain::authentication::value_objects::Identity;

use crate::management::{AuthzStateBody, SetStateValidator};

#[utoipa::path(
    get,
    path = "/authz/state",
    tag = "authz",
    summary = "Get the authorization state of a realm",
    description = "Whether the realm answers authorization questions, and how many times its rules changed. Requires view_authorization.",
    params(("realm_name" = String, Path, description = "Realm name")),
    responses(
        (status = 200, description = "State", body = AuthzStateBody),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
    ),
)]
pub async fn get_state(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<AuthzStateBody>, ApiError> {
    let found = state.service.get_state(identity, realm_name).await?;
    Ok(Response::OK(found.into()))
}

#[utoipa::path(
    put,
    path = "/authz/state",
    tag = "authz",
    summary = "Enable or disable authorization for a realm",
    description = "Requires manage_authorization.",
    params(("realm_name" = String, Path, description = "Realm name")),
    request_body = SetStateValidator,
    responses(
        (status = 200, description = "State", body = AuthzStateBody),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
    ),
)]
pub async fn set_state(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<SetStateValidator>,
) -> Result<Response<AuthzStateBody>, ApiError> {
    let updated = state
        .service
        .set_enabled(identity, realm_name, payload.enabled)
        .await?;
    Ok(Response::Updated(updated.into()))
}

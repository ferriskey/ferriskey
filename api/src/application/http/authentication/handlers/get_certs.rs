use crate::application::http::server::api_entities::api_error::{ApiError, ApiErrorResponse};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::{Path, State};
use ferriskey_core::domain::{authentication::ports::AuthService, jwt::entities::JwkKey};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct GetCertsResponse {
    pub keys: Vec<JwkKey>,
}

async fn fetch_realm_jwks(
    realm_name: String,
    state: AppState,
) -> Result<Response<GetCertsResponse>, ApiError> {
    let jwk_keys = state
        .service
        .get_certs(realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetCertsResponse { keys: jwk_keys }))
}

#[utoipa::path(
    get,
    path = "/protocol/openid-connect/certs",
    tag = "auth",
    summary = "Get JWK keys for a realm",
    description = "Retrieves the JSON Web Key (JWK) keys for a specific realm, used for verifying JWT tokens.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = GetCertsResponse),
        (status = 400, description = "Invalid key", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
pub async fn get_certs(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Response<GetCertsResponse>, ApiError> {
    fetch_realm_jwks(realm_name, state).await
}

#[utoipa::path(
    get,
    path = "/protocol/openid-connect/jwks.json",
    tag = "auth",
    summary = "Get JWKS for a realm",
    description = "Retrieves the JSON Web Key Set (JWKS) for a specific realm, used by resource servers to validate JWT signatures.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "JWKS retrieved successfully", body = GetCertsResponse),
        (status = 400, description = "Invalid key", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
pub async fn get_jwks_json(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Response<GetCertsResponse>, ApiError> {
    fetch_realm_jwks(realm_name, state).await
}

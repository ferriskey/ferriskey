use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::{authentication::ports::AuthService, jwt::entities::JwkKey};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct GetCertsResponse {
    pub keys: Vec<JwkKey>,
}

const KEYS_CACHE_CONTROL: &str = "public, max-age=300";

async fn fetch_realm_jwks(
    realm_name: String,
    state: AppState,
) -> Result<impl IntoResponse, ApiError> {
    let jwk_keys = state
        .service
        .get_certs(realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok((
        [(
            header::CACHE_CONTROL,
            HeaderValue::from_static(KEYS_CACHE_CONTROL),
        )],
        Response::OK(GetCertsResponse { keys: jwk_keys }),
    ))
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
) -> Result<impl IntoResponse, ApiError> {
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
) -> Result<impl IntoResponse, ApiError> {
    fetch_realm_jwks(realm_name, state).await
}

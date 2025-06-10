use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::domain::jwt::entities::jwt::JwkKey;
use crate::domain::jwt::ports::jwt_service::JwtService;
use crate::domain::realm::ports::realm_service::RealmService;
use axum::extract::State;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/protocol/openid-connect/certs")]
pub struct GetCertsRoute {
    realm_name: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[typeshare]
pub struct GetCertsResponse {
    pub keys: Vec<JwkKey>,
}

#[utoipa::path(
    get,
    path = "/protocol/openid-connect/certs",
    tag = "auth",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = GetCertsResponse)
    )
)]
pub async fn get_certs(
    GetCertsRoute { realm_name }: GetCertsRoute,
    State(state): State<AppState>,
) -> Result<Response<GetCertsResponse>, ApiError> {
    let realm = state
        .realm_service
        .get_by_name(realm_name.clone())
        .await
        .map_err(ApiError::from)?;

    let jwt_keypair = state
        .jwt_service
        .retrieve_realm_rsa_keys(&realm)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let jwk_key = jwt_keypair
        .to_jwk_key()
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Response::OK(GetCertsResponse {
        keys: vec![jwk_key],
    }))
}

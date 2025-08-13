use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::domain::jwt::entities::JwkKey;
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
    summary = "Get JWK keys for a realm",
    description = "Retrieves the JSON Web Key (JWK) keys for a specific realm, used for verifying JWT tokens.",
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
    let jwk_keys = state
        .use_case_bundle
        .get_certs_use_case
        .execute(realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetCertsResponse { keys: jwk_keys }))
}

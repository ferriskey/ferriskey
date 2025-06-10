use axum::extract::State;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/protocol/openid-connect/certs")]
pub struct GetCertsRoute {
    realm_name: String
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq)]
#[typeshare]
pub struct GetCertsResponse {

}

#[utoipa::path(
    get,
    path = "/protocol/openid-connect/certs",
    tag = "auth",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Public keys for the realm", body = Response<()>) // Adjust the body type as needed
    )
)]
pub async fn get_certs(
    GetCertsRoute { realm_name }: GetCertsRoute,
    State(state): State<AppState>
) -> Result<Response<>, ApiError> {
    todo!("Implement the logic to retrieve the public keys for the realm: {}", realm_name);
}
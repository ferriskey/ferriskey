use axum::extract::State;
use axum_macros::TypedPath;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    application::http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
    domain::client::{
        entities::redirect_uri::RedirectUri, ports::redirect_uri_service::RedirectUriService,
    },
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients/{client_id}/redirects")]
pub struct GetRedirectUriRoute {
    pub realm_name: String,
    pub client_id: Uuid,
}

#[utoipa::path(
    get,
    path = "/{client_id}/redirects",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, body = Vec<RedirectUri>),
    ),
)]
pub async fn get_redirect_uris(
    GetRedirectUriRoute {
        realm_name,
        client_id,
    }: GetRedirectUriRoute,
    State(state): State<AppState>,
) -> Result<Response<Vec<RedirectUri>>, ApiError> {
    state
        .redirect_uri_service
        .get_by_client_id(client_id)
        .await
        .map_err(ApiError::from)
        .map(|uris| Response::OK(uris))
}

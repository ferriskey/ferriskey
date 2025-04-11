use axum::extract::State;
use axum_macros::TypedPath;
use serde::Deserialize;

use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::domain::realm::{entities::realm::Realm, ports::realm_service::RealmService};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{name}")]
pub struct GetRealmRoute {
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/{name}",
    tag = "realm",
    params(
        ("name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = Realm)
    ),
)]
pub async fn get_realm(
    GetRealmRoute { name }: GetRealmRoute,
    State(state): State<AppState>,
) -> Result<Response<Realm>, ApiError> {
    state
        .realm_service
        .get_by_name(name)
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

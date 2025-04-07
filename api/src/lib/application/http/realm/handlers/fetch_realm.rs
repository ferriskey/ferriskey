use std::sync::Arc;

use axum::Extension;
use axum_macros::TypedPath;
use serde::Deserialize;

use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::domain::realm::{entities::realm::Realm, ports::realm_service::RealmService};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms")]
pub struct GetRealmRoute {}

#[utoipa::path(
    get,
    path = "",
    tag = "realm",
    responses(
        (status = 200, body = Vec<Realm>)
    ),
)]
pub async fn fetch_realm<R: RealmService>(
    _: GetRealmRoute,
    Extension(realm_service): Extension<Arc<R>>,
) -> Result<Response<Vec<Realm>>, ApiError> {
    let realms = realm_service.fetch_realm().await.map_err(ApiError::from)?;

    Ok(Response::OK(realms))
}

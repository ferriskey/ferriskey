use std::sync::Arc;

use axum::Extension;
use axum_macros::TypedPath;
use serde::Deserialize;

use crate::application::http::server::errors::ApiError;
use crate::application::http::server::handlers::Response;
use crate::domain::realm::{entities::realm::Realm, ports::RealmService};

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
pub async fn get_realm<R: RealmService>(
    GetRealmRoute { name }: GetRealmRoute,
    Extension(realm_service): Extension<Arc<R>>,
) -> Result<Response<Realm>, ApiError> {
    realm_service
        .get_by_name(name)
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

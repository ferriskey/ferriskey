use std::sync::Arc;

use axum::{Extension, http::StatusCode};
use axum_macros::TypedPath;
use serde::Deserialize;

use crate::application::http::server::errors::ApiError;
use crate::application::http::server::handlers::ApiSuccess;
use crate::domain::realm::{entities::model::Realm, ports::RealmService};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{name}")]
pub struct GetRealmRoute {
    pub name: String,
}

pub async fn get_realm<R: RealmService>(
    GetRealmRoute { name }: GetRealmRoute,
    Extension(realm_service): Extension<Arc<R>>,
) -> Result<ApiSuccess<Realm>, ApiError> {
    let realm = realm_service
        .get_by_name(name)
        .await
        .map_err(ApiError::from)?;

    Ok(ApiSuccess::new(StatusCode::OK, realm))
}

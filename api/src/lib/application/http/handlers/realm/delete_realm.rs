use std::sync::Arc;

use axum::{Extension, http::StatusCode};
use axum_macros::TypedPath;
use serde::Deserialize;

use crate::{
    application::http::{
        errors::{ApiError, ValidateJson},
        handlers::ApiSuccess,
        validation::realm::DeleteRealmValidator,
    },
    domain::realm::{entities::model::Realm, ports::RealmService},
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{name}")]
pub struct DeleteRealmRoute {
    pub name: String,
}

pub async fn delete_realm<R: RealmService>(
    _: DeleteRealmRoute,
    Extension(realm_service): Extension<Arc<R>>,
    ValidateJson(payload): ValidateJson<DeleteRealmValidator>,
) -> Result<ApiSuccess<Realm>, ApiError> {
    realm_service
        .delete_realm(payload.id)
        .await
        .map_err(ApiError::from)
        .map(|realm| ApiSuccess::new(StatusCode::OK, realm))
}
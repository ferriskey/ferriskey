use std::sync::Arc;

use axum::{Extension, http::StatusCode};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::application::http::server::errors::ApiError;
use crate::application::http::server::handlers::ApiSuccess;
use crate::domain::realm::ports::RealmService;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{name}")]
pub struct DeleteRealmRoute {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteRealmResponse(String);

pub async fn delete_realm<R: RealmService>(
    DeleteRealmRoute { name }: DeleteRealmRoute,
    Extension(realm_service): Extension<Arc<R>>,
) -> Result<ApiSuccess<DeleteRealmResponse>, ApiError> {
    info!("try to delete realm: {}", name);
    realm_service
        .delete_by_name(name)
        .await
        .map_err(ApiError::from)
        .map(|_| {
            ApiSuccess::new(
                StatusCode::OK,
                DeleteRealmResponse("Realm deleted successfully".to_string()),
            )
        })
}

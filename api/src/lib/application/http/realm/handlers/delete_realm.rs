use std::sync::Arc;

use axum::Extension;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::domain::realm::ports::realm_service::RealmService;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{name}")]
pub struct DeleteRealmRoute {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteRealmResponse(String);

#[utoipa::path(
    delete,
    path = "/{name}",
    tag = "realm",
    params(
          ("name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200)
    ),
)]
pub async fn delete_realm<R: RealmService>(
    DeleteRealmRoute { name }: DeleteRealmRoute,
    Extension(realm_service): Extension<Arc<R>>,
) -> Result<Response<String>, ApiError> {
    info!("try to delete realm: {}", name);
    realm_service
        .delete_by_name(name)
        .await
        .map_err(ApiError::from)
        .map(|_| Response::OK("Realm deleted successfully".to_string()))
}

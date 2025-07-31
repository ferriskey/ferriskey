use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::domain::realm::ports::RealmService;
use serde::{Deserialize, Serialize};
use tracing::info;

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
pub async fn delete_realm(
    DeleteRealmRoute { name }: DeleteRealmRoute,
    State(state): State<AppState>,
) -> Result<Response<String>, ApiError> {
    info!("try to delete realm: {}", name);
    state
        .service_bundle
        .realm_service
        .delete_by_name(name)
        .await
        .map_err(ApiError::from)
        .map(|_| Response::OK("Realm deleted successfully".to_string()))
}

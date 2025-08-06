use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::realm::use_cases::delete_realm_use_case::DeleteRealmUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
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
    Extension(identity): Extension<Identity>,
) -> Result<Response<String>, ApiError> {
    info!("try to delete realm: {}", name);
    state
        .use_case_bundle
        .delete_realm_use_case
        .execute(identity, DeleteRealmUseCaseParams { realm_name: name })
        .await
        .map_err(ApiError::from)
        .map(|_| Response::OK("Realm deleted successfully".to_string()))
}

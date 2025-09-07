use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::Path;
use axum::{Extension, extract::State};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::realm::ports::{DeleteRealmInput, RealmService};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DeleteRealmResponse(String);

#[utoipa::path(
    delete,
    path = "/{name}",
    tag = "realm",
    summary = "Delete a realm by name",
    description = "Deletes a realm by its name. This action is irreversible and will remove all associated data.",
    params(
          ("name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = DeleteRealmResponse, description = "Realm deleted successfully"),
    ),
)]
pub async fn delete_realm(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<String>, ApiError> {
    info!("try to delete realm: {}", name);
    state
        .service
        .delete_realm(identity, DeleteRealmInput { realm_name: name })
        .await
        .map_err(ApiError::from)
        .map(|_| Response::OK("Realm deleted successfully".to_string()))
}

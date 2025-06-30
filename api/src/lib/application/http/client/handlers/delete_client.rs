use crate::application::auth::Identity;
use crate::application::http::client::policies::ClientPolicy;
use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::domain::client::ports::client_service::ClientService;
use crate::domain::realm::ports::realm_service::RealmService;
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use tracing::info;
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients/{client_id}")]
pub struct DeleteClientRoute {
    pub realm_name: String,
    pub client_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct DeleteClientResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/{client_id}",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, body = DeleteClientResponse, description = "Client deleted successfully"),
    ),
)]
pub async fn delete_client(
    DeleteClientRoute {
        realm_name,
        client_id,
    }: DeleteClientRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteClientResponse>, ApiError> {
    info!(
        "Deleting client with ID {} in realm {}",
        client_id, realm_name
    );
    let realm = state
        .realm_service
        .get_by_name(realm_name.clone())
        .await
        .map_err(ApiError::from)?;

    if !ClientPolicy::delete(identity, state.clone(), realm).await? {
        return Err(ApiError::Forbidden(
            "You do not have permission to delete this client".to_string(),
        ));
    }

    info!("can delete");

    state
        .client_service
        .delete_by_id(client_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeleteClientResponse {
        message: format!("Client with ID {client_id} in realm {realm_name} deleted successfully"),
    }))
}

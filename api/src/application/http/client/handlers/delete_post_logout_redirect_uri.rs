use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::{
    entities::DeletePostLogoutRedirectUriInput, ports::ClientService,
};
use tracing::info;
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/{client_id}/post-logout-redirects/{uri_id}",
    summary = "Delete a post-logout redirect URI for a client",
    description = "Deletes a specific post-logout redirect URI for a client in a realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("uri_id" = Uuid, Path, description = "Post-logout redirect URI ID"),
    ),
    tag = "client",
    responses(
        (status = 200, description = "Post-logout redirect URI deleted successfully"),
    ),
)]
pub async fn delete_post_logout_redirect_uri(
    Path((realm_name, client_id, uri_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    info!(
        "Deleting post-logout redirect URI: realm_name={}, client_id={}, uri_id={}",
        realm_name, client_id, uri_id
    );

    state
        .service
        .delete_post_logout_redirect_uri(
            identity,
            DeletePostLogoutRedirectUriInput {
                client_id,
                uri_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(|_| Response::OK(()))
}

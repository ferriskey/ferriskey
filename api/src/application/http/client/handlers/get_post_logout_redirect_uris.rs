use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::redirect_uri::RedirectUri;
use ferriskey_core::domain::client::{
    entities::GetPostLogoutRedirectUrisInput, ports::ClientService,
};
use tracing::info;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{client_id}/post-logout-redirects",
    summary = "Get post-logout redirect URIs for a client",
    description = "Retrieves all post-logout redirect URIs associated with a client in a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, body = Vec<RedirectUri>),
    ),
)]
pub async fn get_post_logout_redirect_uris(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<RedirectUri>>, ApiError> {
    info!(
        "Fetching post-logout redirect URIs for client: realm_name={}, client_id={}",
        realm_name, client_id
    );

    state
        .service
        .get_post_logout_redirect_uris(
            identity,
            GetPostLogoutRedirectUrisInput {
                client_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}

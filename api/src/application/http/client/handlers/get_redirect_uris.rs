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
use ferriskey_core::domain::client::{entities::GetRedirectUrisInput, ports::ClientService};
use tracing::info;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{client_id}/redirects",
    summary = "Get redirect URIs for a client",
    description = "Retrieves all redirect URIs associated with a client in a specific realm. This endpoint is useful for OAuth2 or OpenID Connect flows where clients need to know their registered redirect URIs.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, body = Vec<RedirectUri>),
    ),
)]
pub async fn get_redirect_uris(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<RedirectUri>>, ApiError> {
    info!(
        "Fetching redirect URIs for client: realm_name={}, client_id={}",
        realm_name, client_id
    );

    state
        .service
        .get_redirect_uris(
            identity,
            GetRedirectUrisInput {
                client_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}

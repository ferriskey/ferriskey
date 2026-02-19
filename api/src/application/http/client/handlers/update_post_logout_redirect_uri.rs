use crate::application::http::{
    client::validators::UpdateRedirectUriValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::redirect_uri::RedirectUri;
use ferriskey_core::domain::client::{
    entities::UpdatePostLogoutRedirectUriInput, ports::ClientService,
};
use tracing::info;
use uuid::Uuid;

#[utoipa::path(
    put,
    path = "/{client_id}/post-logout-redirects/{uri_id}",
    summary = "Update a post-logout redirect URI for a client",
    description = "Updates an existing post-logout redirect URI for a client in a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("uri_id" = Uuid, Path, description = "Post-logout redirect URI ID"),
    ),
    tag = "client",
    request_body = UpdateRedirectUriValidator,
    responses(
        (status = 200, body = RedirectUri),
    ),
)]
pub async fn update_post_logout_redirect_uri(
    Path((realm_name, client_id, uri_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateRedirectUriValidator>,
) -> Result<Response<RedirectUri>, ApiError> {
    info!(
        "Updating post-logout redirect URI: realm_name={}, client_id={}, uri_id={}",
        realm_name, client_id, uri_id
    );
    state
        .service
        .update_post_logout_redirect_uri(
            identity,
            UpdatePostLogoutRedirectUriInput {
                redirect_uri_id: uri_id,
                realm_name,
                client_id,
                enabled: payload.enabled,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::Updated)
}

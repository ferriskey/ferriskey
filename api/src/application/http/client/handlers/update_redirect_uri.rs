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
use ferriskey_core::domain::client::{entities::UpdateRedirectUriInput, ports::ClientService};
use tracing::info;
use uuid::Uuid;

#[utoipa::path(
    put,
    path = "/{client_id}/redirects/{uri_id}",
    summary = "Update a redirect URI for a client",
    description = "Updates an existing redirect URI for a client in a specific realm. This endpoint allows you to modify the enabled status of a redirect URI.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("uri_id" = Uuid, Path, description = "Redirect URI ID"),
    ),
    tag = "client",
    request_body = UpdateRedirectUriValidator,
    responses(
        (status = 200, body = RedirectUri),
    ),
)]
pub async fn update_redirect_uri(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    Path(uri_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateRedirectUriValidator>,
) -> Result<Response<RedirectUri>, ApiError> {
    info!(
        "Updating redirect URI: realm_name={}, client_id={}, uri_id={}",
        realm_name, client_id, uri_id
    );
    state
        .service
        .update_redirect_uri(
            identity,
            UpdateRedirectUriInput {
                redirect_uri_id: uri_id,
                realm_name,
                client_id,
                enabled: payload.enabled,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}

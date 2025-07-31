use crate::application::http::{
    client::{
        routes::client_routes::UpdateRedirectUriRoute, validators::UpdateRedirectUriValidator,
    },
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::extract::State;
use ferriskey_core::domain::client::entities::redirect_uri::RedirectUri;
use ferriskey_core::domain::client::ports::RedirectUriService;
use tracing::info;

#[utoipa::path(
    put,
    path = "/{client_id}/redirects/{uri_id}",
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
    UpdateRedirectUriRoute {
        realm_name,
        client_id,
        uri_id,
    }: UpdateRedirectUriRoute,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<UpdateRedirectUriValidator>,
) -> Result<Response<RedirectUri>, ApiError> {
    info!(
        "Updating redirect URI: realm_name={}, client_id={}, uri_id={}",
        realm_name, client_id, uri_id
    );
    state
        .service_bundle
        .redirect_uri_service
        .update_enabled(uri_id, payload.enabled)
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}

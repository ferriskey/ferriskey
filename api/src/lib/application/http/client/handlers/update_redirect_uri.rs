use axum::extract::State;
use axum_macros::TypedPath;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    application::http::{
        client::validators::UpdateRedirectUriValidator,
        server::{
            api_entities::{
                api_error::{ApiError, ValidateJson},
                response::Response,
            },
            app_state::AppState,
        },
    },
    domain::client::{
        entities::redirect_uri::RedirectUri, ports::redirect_uri_service::RedirectUriService,
    },
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}")]
pub struct CreateRedirectUriRoute {
    pub realm_name: String,
    pub client_id: Uuid,
    pub uri_id: Uuid,
}

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
    CreateRedirectUriRoute {
        realm_name,
        client_id,
        uri_id,
    }: CreateRedirectUriRoute,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<UpdateRedirectUriValidator>,
) -> Result<Response<RedirectUri>, ApiError> {
    state
        .redirect_uri_service
        .update_enabled(uri_id, payload.enabled)
        .await
        .map_err(ApiError::from)
        .map(|uri| Response::OK(uri))
}

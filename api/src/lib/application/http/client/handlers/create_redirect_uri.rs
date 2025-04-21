use axum::extract::State;
use axum_macros::TypedPath;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    application::http::{
        client::validators::CreateRedirectUriValidator,
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
#[typed_path("/realms/{realm_name}/clients/{client_id}/redirects")]
pub struct CreateRedirectUriRoute {
    pub realm_name: String,
    pub client_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/{client_id}/redirects",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    request_body = CreateRedirectUriValidator,
)]
pub async fn create_redirect_uri(
    CreateRedirectUriRoute {
        realm_name,
        client_id,
    }: CreateRedirectUriRoute,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<CreateRedirectUriValidator>,
) -> Result<Response<RedirectUri>, ApiError> {
    state
        .redirect_uri_service
        .add_redirect_uri(payload, realm_name, client_id)
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

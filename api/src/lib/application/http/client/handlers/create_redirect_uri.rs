use axum::extract::State;
use axum_macros::TypedPath;
use serde::Deserialize;

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
#[typed_path("/realms/{realm_name}/clients/{client_id}/redirections")]
pub struct CreateRedirectUriRoute {
    pub realm_name: String,
    pub client_id: String,
}

#[utoipa::path(
    post,
    path = "",
    tag = "redirect_uri",
    request_body = CreateRedirectUriValidator,
)]
pub async fn create_client(
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

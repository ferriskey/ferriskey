use crate::application::http::{
    client::validators::CreateRedirectUriValidator,
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
use ferriskey_core::domain::client::value_objects::CreateRedirectUriRequest;
use ferriskey_core::domain::client::{
    entities::CreatePostLogoutRedirectUriInput, ports::ClientService,
};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/{client_id}/post-logout-redirects",
    summary = "Create a new post-logout redirect URI for a client",
    description = "Creates a new post-logout redirect URI for the specified client.",
    responses(
        (status = 201, body = RedirectUri, description = "Post-logout redirect URI created successfully for the client"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    request_body = CreateRedirectUriValidator,
)]
pub async fn create_post_logout_redirect_uri(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateRedirectUriValidator>,
) -> Result<Response<RedirectUri>, ApiError> {
    state
        .service
        .create_post_logout_redirect_uri(
            identity,
            CreatePostLogoutRedirectUriInput {
                client_id,
                payload: CreateRedirectUriRequest {
                    enabled: payload.enabled,
                    value: payload.value,
                },
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

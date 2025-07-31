use crate::application::http::{
    client::{
        routes::client_routes::CreateRedirectUriRoute, validators::CreateRedirectUriValidator,
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
use ferriskey_core::domain::client::value_objects::CreateRedirectUriRequest;

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
        .service_bundle
        .redirect_uri_service
        .add_redirect_uri(
            CreateRedirectUriRequest {
                enabled: payload.enabled,
                value: payload.value,
            },
            realm_name,
            client_id,
        )
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

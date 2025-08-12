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
use axum::Extension;
use axum::extract::State;
use ferriskey_core::application::client::use_cases::create_redirect_uri_use_case::CreateRedirectUriUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::redirect_uri::RedirectUri;
use ferriskey_core::domain::client::value_objects::CreateRedirectUriRequest;

#[utoipa::path(
    post,
    path = "/{client_id}/redirects",
    summary = "Create a new redirect URI for a client",
    description = "Creates a new redirect URI for the specified client. This endpoint allows you to add a redirect URI that the client can use for OAuth2 or OpenID Connect flows.",
    responses(
        (status = 201, body = RedirectUri, description = "Redirect URI created successfully for the client"),
    ),
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
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateRedirectUriValidator>,
) -> Result<Response<RedirectUri>, ApiError> {
    state
        .use_case_bundle
        .create_redirect_uri_use_case
        .execute(
            identity,
            CreateRedirectUriUseCaseParams {
                client_id,
                realm_name,
                payload: CreateRedirectUriRequest {
                    value: payload.value,
                    enabled: payload.enabled,
                },
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

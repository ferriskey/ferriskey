use crate::application::http::{
    client::routes::client_routes::GetRedirectUriRoute,
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use axum::Extension;
use axum::extract::State;
use ferriskey_core::application::client::use_cases::get_redirect_uris_use_case::GetRedirectUrisUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::redirect_uri::RedirectUri;
use tracing::info;

#[utoipa::path(
    get,
    path = "/{client_id}/redirects",
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
    GetRedirectUriRoute {
        realm_name,
        client_id,
    }: GetRedirectUriRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<RedirectUri>>, ApiError> {
    info!(
        "Fetching redirect URIs for client: realm_name={}, client_id={}",
        realm_name, client_id
    );

    state
        .use_case_bundle
        .get_redirect_uris_use_case
        .execute(
            identity,
            GetRedirectUrisUseCaseParams {
                client_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}

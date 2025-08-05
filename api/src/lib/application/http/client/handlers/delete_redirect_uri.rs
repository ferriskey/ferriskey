use crate::application::http::{
    client::routes::client_routes::DeleteRedirectUriRoute,
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use axum::Extension;
use axum::extract::State;
use ferriskey_core::application::client::use_cases::delete_redirect_uri_use_case::DeleteRedirectUriUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use tracing::info;

#[utoipa::path(
    delete,
    path = "/{client_id}/redirects/{uri_id}",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("uri_id" = Uuid, Path, description = "Redirect URI ID"),
    ),
    tag = "client",
    responses(
        (status = 200, description = "Redirect URI deleted successfully"),
    ),
)]
pub async fn delete_redirect_uri(
    DeleteRedirectUriRoute {
        realm_name,
        client_id,
        uri_id,
    }: DeleteRedirectUriRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    info!(
        "Deleting redirect URI: realm_name={}, client_id={}, uri_id={}",
        realm_name, client_id, uri_id
    );
    state
        .use_case_bundle
        .delete_redirect_uri_use_case
        .execute(
            identity,
            DeleteRedirectUriUseCaseParams {
                uri_id,
                client_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(|_| Response::OK(()))
}

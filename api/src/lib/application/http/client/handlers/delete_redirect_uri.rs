use axum::extract::State;

use crate::{
    application::http::{
        client::routes::client_routes::DeleteRedirectUriRoute,
        server::{
            api_entities::{api_error::ApiError, response::Response},
            app_state::AppState,
        },
    },
    domain::client::ports::redirect_uri_service::RedirectUriService,
};

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
) -> Result<Response<()>, ApiError> {
    state
        .redirect_uri_service
        .delete(uri_id)
        .await
        .map_err(ApiError::from)
        .map(|_| Response::OK(()))
}

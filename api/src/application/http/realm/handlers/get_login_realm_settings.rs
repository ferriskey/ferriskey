use axum::extract::{Path, State};
use ferriskey_core::domain::realm::{entities::RealmLoginSetting, ports::RealmService};

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[utoipa::path(
    get,
    path = "/{name}/login-settings",
    tag = "realm",
    summary = "Get login settings",
    description = "Get the login settings for a specific realm.",
    params(
        ("name" = String, Path, description = "The name of the realm"),
    ),
    responses(
        (status = 200, body = RealmLoginSetting)
    )
)]
pub async fn get_login_realm_settings_handler(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Response<RealmLoginSetting>, ApiError> {
    state
        .service
        .get_login_settings(realm_name)
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

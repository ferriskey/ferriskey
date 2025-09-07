use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    realm::{
        entities::RealmSetting,
        ports::{GetRealmSettingInput, RealmService},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct RealmSettingResponse {
    pub data: RealmSetting,
}

#[utoipa::path(
    get,
    path = "/{realm_name}/users/@me/realms/settings",
    tag = "realm",
    summary = "Get user realm settings",
    description = "Retrieves the settings of the specified realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    security(
        ("Authorization" = ["Bearer"]),
    ),
    responses(
        (status = 200, body = RealmSettingResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
)]
pub async fn get_user_realm_settings(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<RealmSetting>, ApiError> {
    state
        .service
        .get_realm_setting_by_name(identity, GetRealmSettingInput { realm_name })
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

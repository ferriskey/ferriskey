use crate::application::http::realm::validators::UpdateRealmSettingValidator;

use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;

use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::domain::realm::entities::{Realm, RealmSetting};
use ferriskey_core::domain::realm::ports::RealmService;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{name}/settings")]
pub struct UpdateRealmSettingsRoute {
    pub name: String,
}

#[utoipa::path(
    put,
    path = "/{name}/settings",
    tag = "realm",
    params(
        ("name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = Realm)
    ),
    request_body = UpdateRealmSettingValidator
)]
pub async fn update_realm_setting(
    UpdateRealmSettingsRoute { name }: UpdateRealmSettingsRoute,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<UpdateRealmSettingValidator>,
) -> Result<Response<RealmSetting>, ApiError> {
    let realm = state
        .service_bundle
        .realm_service
        .get_by_name(name)
        .await
        .map_err(ApiError::from)?;

    state
        .service_bundle
        .realm_service
        .update_realm_setting(realm.id, payload.default_signing_algorithm)
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

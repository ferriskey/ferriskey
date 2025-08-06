use crate::application::http::realm::validators::UpdateRealmSettingValidator;
use axum::Extension;

use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;

use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::realm::use_cases::update_realm_settings_use_case::UpdateRealmSettingsUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::realm::entities::{Realm, RealmSetting};
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
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateRealmSettingValidator>,
) -> Result<Response<RealmSetting>, ApiError> {
    state
        .use_case_bundle
        .update_realm_settings_use_case
        .execute(
            identity,
            UpdateRealmSettingsUseCaseParams {
                realm_name: name,
                algorithm: payload.default_signing_algorithm,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

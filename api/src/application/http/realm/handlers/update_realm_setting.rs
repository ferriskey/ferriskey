use crate::application::http::realm::validators::UpdateRealmSettingValidator;
use axum::Extension;
use ferriskey_core::domain::realm::ports::{RealmService, UpdateRealmSettingInput};

use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;

use axum::extract::{Path, State};

use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::realm::entities::Realm;

#[utoipa::path(
    put,
    path = "/{name}/settings",
    tag = "realm",
    summary = "Update settings for a realm by name",
    description = "Updates the settings for a specified realm. This endpoint allows modification of various realm settings.",
    params(
        ("name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = Realm)
    ),
    request_body = UpdateRealmSettingValidator
)]
pub async fn update_realm_setting(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateRealmSettingValidator>,
) -> Result<Response<Realm>, ApiError> {
    state
        .service
        .update_realm_setting(
            identity,
            UpdateRealmSettingInput {
                realm_name: name,
                algorithm: payload.default_signing_algorithm,

                forgot_password_enabled: payload.forgot_password_enabled,
                remember_me_enabled: payload.remember_me_enabled,
                user_registration_enabled: payload.user_registration_enabled,
                magic_link_enabled: payload.magic_link_enabled,
                magic_link_ttl_minutes: payload.magic_link_ttl_minutes,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

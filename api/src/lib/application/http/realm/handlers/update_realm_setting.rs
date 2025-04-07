use std::sync::Arc;

use crate::application::http::realm::validators::UpdateRealmSettingValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::domain::realm::entities::realm_setting::RealmSetting;
use crate::domain::realm::{entities::realm::Realm, ports::realm_service::RealmService};
use axum::Extension;
use axum_macros::TypedPath;
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
pub async fn update_realm_setting<R: RealmService>(
    UpdateRealmSettingsRoute { name }: UpdateRealmSettingsRoute,
    Extension(realm_service): Extension<Arc<R>>,
    ValidateJson(payload): ValidateJson<UpdateRealmSettingValidator>,
) -> Result<Response<RealmSetting>, ApiError> {
    let realm = realm_service.get_by_name(name).await?;

    realm_service
        .update_realm_setting(realm.id, payload.default_signing_algorithm)
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

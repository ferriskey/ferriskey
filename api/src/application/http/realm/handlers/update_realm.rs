use crate::application::http::realm::validators::UpdateRealmValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::realm::use_cases::update_realm_use_case::UpdateRealmUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::realm::entities::Realm;

#[utoipa::path(
    put,
    path = "/{name}",
    tag = "realm",
    summary = "Update a realm by name",
    description = "Updates the name of an existing realm. This endpoint allows you to change the name of a realm while keeping its associated data intact.",
    params(
        ("name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = Realm)
    ),
    request_body = UpdateRealmValidator
)]
pub async fn update_realm(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateRealmValidator>,
) -> Result<Response<Realm>, ApiError> {
    state
        .use_case_bundle
        .update_realm_use_case
        .execute(
            identity,
            UpdateRealmUseCaseParams {
                realm_name: name,
                new_realm_name: payload.name,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

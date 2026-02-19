use crate::application::http::realm::validators::UpdateRealmValidator;
use crate::application::http::server::api_entities::api_error::{
    ApiError, ApiErrorResponse, ValidateJson,
};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::realm::entities::Realm;
use ferriskey_core::domain::realm::ports::{RealmService, UpdateRealmInput};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateRealmResponse {
    pub data: Realm,
}

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
        (status = 200, description = "Realm updated successfully", body = UpdateRealmResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    request_body = UpdateRealmValidator
)]
pub async fn update_realm(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateRealmValidator>,
) -> Result<Response<UpdateRealmResponse>, ApiError> {
    let realm = state
        .service
        .update_realm(
            identity,
            UpdateRealmInput {
                realm_name: name,
                name: payload.name,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateRealmResponse { data: realm }))
}

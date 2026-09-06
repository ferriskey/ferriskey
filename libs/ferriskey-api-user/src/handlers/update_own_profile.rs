use crate::validators::UpdateOwnProfileValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::User;
use ferriskey_core::domain::user::{entities::UpdateOwnProfileInput, ports::UserService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateOwnProfileResponse {
    pub data: User,
}

#[utoipa::path(
    put,
    path = "/me",
    tag = "user",
    summary = "Update the caller's own profile",
    description = "Updates firstname, lastname and email of the currently authenticated user. Username can only be changed when the realm has username editing enabled.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body(
        content = UpdateOwnProfileValidator,
        description = "Profile fields to update",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Profile updated successfully", body = UpdateOwnProfileResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions, or username editing is disabled", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn update_own_profile(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateOwnProfileValidator>,
) -> Result<Response<UpdateOwnProfileResponse>, ApiError> {
    let user = state
        .service
        .update_own_profile(
            identity,
            UpdateOwnProfileInput {
                realm_name,
                username: payload.username,
                firstname: payload.firstname,
                lastname: payload.lastname,
                email: payload.email,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateOwnProfileResponse { data: user }))
}

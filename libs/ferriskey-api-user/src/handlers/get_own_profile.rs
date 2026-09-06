use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::user::entities::User;
use ferriskey_core::domain::user::ports::UserService;
use ferriskey_core::domain::{
    authentication::value_objects::Identity, user::entities::GetOwnProfileInput,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct OwnProfileResponse {
    pub data: User,
}

#[utoipa::path(
    get,
    path = "/me",
    tag = "user",
    summary = "Get the caller's own profile",
    description = "Retrieves the profile of the currently authenticated user. The user id is taken from the access token, never from the URL.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Profile retrieved successfully", body = OwnProfileResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_own_profile(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<OwnProfileResponse>, ApiError> {
    let user = state
        .service
        .get_own_profile(identity, GetOwnProfileInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(OwnProfileResponse { data: user }))
}

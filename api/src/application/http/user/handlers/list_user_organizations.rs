use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    organization::ports::{ListUserOrganizationsInput, OrganizationMember, OrganizationService},
};
use uuid::Uuid;

use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    get,
    path = "/{user_id}/organizations",
    tag = "user",
    summary = "List organizations for a user",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "Organizations retrieved successfully", body = Vec<OrganizationMember>),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_user_organizations(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<OrganizationMember>>, ApiError> {
    state
        .service
        .list_user_organizations(
            identity,
            ListUserOrganizationsInput {
                realm_name,
                user_id,
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

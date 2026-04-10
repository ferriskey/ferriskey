use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    organization::ports::{OrganizationId, OrganizationService, RemoveOrganizationMemberInput},
};
use uuid::Uuid;

use crate::application::http::server::api_entities::api_error::{ApiError, ApiErrorResponse};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    delete,
    path = "/{organization_id}/members/{user_id}",
    tag = "organization",
    summary = "Remove a member from an organization",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "Member removed successfully"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization or member not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn remove_member(
    Path((realm_name, organization_id, user_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .service
        .remove_member(
            identity,
            RemoveOrganizationMemberInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                user_id,
            },
        )
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(ApiError::from)
}

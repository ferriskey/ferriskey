use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    organization::ports::{
        AddOrganizationMemberInput, OrganizationId, OrganizationMember, OrganizationService,
    },
};
use uuid::Uuid;

use crate::application::http::organization::validators::AddMemberValidator;
use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    post,
    path = "/{organization_id}/members",
    tag = "organization",
    summary = "Add a member to an organization",
    request_body = AddMemberValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
    ),
    responses(
        (status = 201, description = "Member added successfully", body = OrganizationMember),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization or user not found", body = ApiErrorResponse),
        (status = 409, description = "User is already a member", body = ApiErrorResponse),
        (status = 422, description = "Validation error", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn add_member(
    Path((realm_name, organization_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<AddMemberValidator>,
) -> Result<Response<OrganizationMember>, ApiError> {
    state
        .service
        .add_member(
            identity,
            AddOrganizationMemberInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                user_id: payload.user_id,
            },
        )
        .await
        .map(Response::Created)
        .map_err(ApiError::from)
}

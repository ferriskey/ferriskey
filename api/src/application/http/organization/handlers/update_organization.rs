use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    organization::ports::{
        Organization, OrganizationId, OrganizationService, UpdateOrganizationInput,
    },
};
use uuid::Uuid;

use crate::application::http::organization::validators::UpdateOrganizationValidator;
use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    put,
    path = "/{organization_id}",
    tag = "organization",
    summary = "Update organization details",
    request_body = UpdateOrganizationValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "Organization updated successfully", body = Organization),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization not found", body = ApiErrorResponse),
        (status = 422, description = "Validation error", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_organization(
    Path((realm_name, organization_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateOrganizationValidator>,
) -> Result<Response<Organization>, ApiError> {
    state
        .service
        .update_organization(
            identity,
            UpdateOrganizationInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                name: payload.name,
                alias: payload.alias,
                domain: payload.domain,
                redirect_url: payload.redirect_url,
                description: payload.description,
                enabled: payload.enabled,
            },
        )
        .await
        .map(Response::Updated)
        .map_err(ApiError::from)
}

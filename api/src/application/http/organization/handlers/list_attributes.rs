use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    organization::ports::{
        ListOrganizationAttributesInput, OrganizationAttribute, OrganizationId, OrganizationService,
    },
};
use uuid::Uuid;

use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    get,
    path = "/{organization_id}/attributes",
    tag = "organization",
    summary = "List organization attributes",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
    ),
    responses(
        (status = 200, description = "Attributes retrieved successfully", body = Vec<OrganizationAttribute>),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_attributes(
    Path((realm_name, organization_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<OrganizationAttribute>>, ApiError> {
    state
        .service
        .list_attributes(
            identity,
            ListOrganizationAttributesInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    organization::ports::{DeleteOrganizationAttributeInput, OrganizationId, OrganizationService},
};
use uuid::Uuid;

use crate::application::http::server::api_entities::api_error::{ApiError, ApiErrorResponse};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    delete,
    path = "/{organization_id}/attributes/{key}",
    tag = "organization",
    summary = "Delete an organization attribute",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("key" = String, Path, description = "Attribute key"),
    ),
    responses(
        (status = 204, description = "Attribute deleted successfully"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization or attribute not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_attribute(
    Path((realm_name, organization_id, key)): Path<(String, Uuid, String)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .service
        .delete_attribute(
            identity,
            DeleteOrganizationAttributeInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                key,
            },
        )
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(ApiError::from)
}

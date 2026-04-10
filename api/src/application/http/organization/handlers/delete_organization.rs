use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    organization::ports::{DeleteOrganizationInput, OrganizationId, OrganizationService},
};
use uuid::Uuid;

use crate::application::http::server::api_entities::api_error::{ApiError, ApiErrorResponse};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    delete,
    path = "/{organization_id}",
    tag = "organization",
    summary = "Delete an organization",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
    ),
    responses(
        (status = 204, description = "Organization deleted successfully"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_organization(
    Path((realm_name, organization_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .service
        .delete_organization(
            identity,
            DeleteOrganizationInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
            },
        )
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(ApiError::from)
}

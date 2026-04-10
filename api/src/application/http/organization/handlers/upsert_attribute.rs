use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    organization::ports::{
        OrganizationAttribute, OrganizationId, OrganizationService,
        UpsertOrganizationAttributeInput,
    },
};
use uuid::Uuid;

use crate::application::http::organization::validators::UpsertAttributeValidator;
use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    put,
    path = "/{organization_id}/attributes/{key}",
    tag = "organization",
    summary = "Create or update an organization attribute",
    request_body = UpsertAttributeValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        ("key" = String, Path, description = "Attribute key"),
    ),
    responses(
        (status = 200, description = "Attribute upserted successfully", body = OrganizationAttribute),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization not found", body = ApiErrorResponse),
        (status = 422, description = "Validation error", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn upsert_attribute(
    Path((realm_name, organization_id, key)): Path<(String, Uuid, String)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpsertAttributeValidator>,
) -> Result<Response<OrganizationAttribute>, ApiError> {
    state
        .service
        .upsert_attribute(
            identity,
            UpsertOrganizationAttributeInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
                key,
                value: payload.value,
            },
        )
        .await
        .map(Response::Updated)
        .map_err(ApiError::from)
}

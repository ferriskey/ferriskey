use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    organization::ports::{CreateOrganizationInput, Organization, OrganizationService},
};

use crate::application::http::organization::validators::CreateOrganizationValidator;
use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use crate::application::http::server::app_state::AppState;

#[utoipa::path(
    post,
    path = "",
    tag = "organization",
    summary = "Create a new organization",
    request_body = CreateOrganizationValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 201, description = "Organization created successfully", body = Organization),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 422, description = "Validation error", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn create_organization(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateOrganizationValidator>,
) -> Result<Response<Organization>, ApiError> {
    state
        .service
        .create_organization(
            identity,
            CreateOrganizationInput {
                realm_name,
                name: payload.name,
                alias: payload.alias,
                domain: payload.domain,
                redirect_url: payload.redirect_url,
                description: payload.description,
                enabled: payload.enabled,
            },
        )
        .await
        .map(Response::Created)
        .map_err(ApiError::from)
}

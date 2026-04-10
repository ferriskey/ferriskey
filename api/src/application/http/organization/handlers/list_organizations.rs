use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    organization::ports::{ListOrganizationsInput, Organization, OrganizationService},
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use crate::application::http::server::app_state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct ListOrganizationsResponse {
    pub data: Vec<Organization>,
}

#[utoipa::path(
    get,
    path = "",
    tag = "organization",
    summary = "List organizations in a realm",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Organizations retrieved successfully", body = ListOrganizationsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_organizations(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ListOrganizationsResponse>, ApiError> {
    state
        .service
        .list_organizations(identity, ListOrganizationsInput { realm_name })
        .await
        .map(|data| Response::OK(ListOrganizationsResponse { data }))
        .map_err(ApiError::from)
}

use ferriskey_core::domain::portal_theme::{
    entities::PortalPageType,
    validation::{REQUIRED_BLOCKS, required_blocks_for},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PageRequirement {
    pub page_type: PortalPageType,
    pub required_blocks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PageRequirementsResponse {
    pub data: Vec<PageRequirement>,
}

#[utoipa::path(
    get,
    path = "/page-requirements",
    tag = "portal-theme",
    summary = "Get portal page block requirements",
    description = "Returns the list of block types that must appear in each page's component tree to pass validation.",
    responses(
        (status = 200, description = "Requirements retrieved successfully", body = PageRequirementsResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_page_requirements() -> Result<Response<PageRequirementsResponse>, ApiError> {
    let data = REQUIRED_BLOCKS
        .iter()
        .map(|(pt, _)| PageRequirement {
            page_type: *pt,
            required_blocks: required_blocks_for(*pt)
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        })
        .collect();

    Ok(Response::OK(PageRequirementsResponse { data }))
}

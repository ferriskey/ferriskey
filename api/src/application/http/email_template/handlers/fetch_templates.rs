use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    email_template::{
        entities::EmailTemplate,
        ports::{EmailTemplateService, GetEmailTemplatesInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetEmailTemplatesResponse {
    pub data: Vec<EmailTemplate>,
}

#[utoipa::path(
    get,
    path = "",
    tag = "email-template",
    summary = "Fetch all email templates",
    description = "Retrieves all email templates for the specified realm.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Email templates retrieved successfully", body = GetEmailTemplatesResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn fetch_templates(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetEmailTemplatesResponse>, ApiError> {
    let templates = state
        .service
        .get_templates_by_realm(identity, GetEmailTemplatesInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetEmailTemplatesResponse { data: templates }))
}

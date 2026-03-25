use axum::extract::Path;
use ferriskey_core::domain::email_template::entities::{EmailType, TemplateVariable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::server::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetTemplateVariablesResponse {
    pub data: Vec<TemplateVariable>,
}

#[utoipa::path(
    get,
    path = "/{email_type}",
    tag = "email-template-variables",
    summary = "Get template variables",
    description = "Returns the available variables for a given email type.",
    params(
        ("email_type" = String, Path, description = "Email type (reset_password, magic_link, email_verification)"),
    ),
    responses(
        (status = 200, description = "Variables retrieved successfully", body = GetTemplateVariablesResponse),
        (status = 400, description = "Invalid email type", body = ApiErrorResponse),
    ),
)]
pub async fn get_variables(
    Path(email_type): Path<String>,
) -> Result<Response<GetTemplateVariablesResponse>, ApiError> {
    let email_type = EmailType::try_from(email_type).map_err(ApiError::from)?;

    Ok(Response::OK(GetTemplateVariablesResponse {
        data: email_type.available_variables(),
    }))
}

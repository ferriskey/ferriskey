use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    email_template::{
        entities::EmailTemplate,
        ports::{EmailTemplateService, GetEmailTemplateInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetEmailTemplateResponse {
    pub data: EmailTemplate,
}

#[utoipa::path(
    get,
    path = "/{template_id}",
    tag = "email-template",
    summary = "Get email template",
    description = "Retrieves a single email template by ID.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("template_id" = Uuid, Path, description = "Email template ID"),
    ),
    responses(
        (status = 200, description = "Email template retrieved successfully", body = GetEmailTemplateResponse),
        (status = 404, description = "Email template not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_template(
    Path((realm_name, template_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetEmailTemplateResponse>, ApiError> {
    let template = state
        .service
        .get_template(
            identity,
            GetEmailTemplateInput {
                realm_name,
                template_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetEmailTemplateResponse { data: template }))
}

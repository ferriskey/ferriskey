use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    email_template::{
        entities::{EmailTemplate, EmailType},
        ports::{CreateEmailTemplateInput, EmailTemplateService},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::{
    email_template::validators::CreateEmailTemplateValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CreateEmailTemplateResponse {
    pub data: EmailTemplate,
}

#[utoipa::path(
    post,
    path = "",
    tag = "email-template",
    summary = "Create email template",
    description = "Creates a new email template for the specified realm.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    request_body = CreateEmailTemplateValidator,
    responses(
        (status = 201, description = "Email template created successfully", body = CreateEmailTemplateResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn create_template(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateEmailTemplateValidator>,
) -> Result<Response<CreateEmailTemplateResponse>, ApiError> {
    let email_type = EmailType::try_from(payload.email_type).map_err(ApiError::from)?;

    let template = state
        .service
        .create_template(
            identity,
            CreateEmailTemplateInput {
                realm_name,
                name: payload.name,
                email_type,
                structure: payload.structure,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Created(CreateEmailTemplateResponse {
        data: template,
    }))
}

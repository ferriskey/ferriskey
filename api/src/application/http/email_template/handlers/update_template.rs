use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    email_template::{
        entities::EmailTemplate,
        ports::{EmailTemplateService, UpdateEmailTemplateInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::{
    email_template::validators::UpdateEmailTemplateValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateEmailTemplateResponse {
    pub data: EmailTemplate,
}

#[utoipa::path(
    put,
    path = "/{template_id}",
    tag = "email-template",
    summary = "Update email template",
    description = "Updates an existing email template.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("template_id" = Uuid, Path, description = "Email template ID"),
    ),
    request_body = UpdateEmailTemplateValidator,
    responses(
        (status = 200, description = "Email template updated successfully", body = UpdateEmailTemplateResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 404, description = "Email template not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_template(
    Path((realm_name, template_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateEmailTemplateValidator>,
) -> Result<Response<UpdateEmailTemplateResponse>, ApiError> {
    let template = state
        .service
        .update_template(
            identity,
            UpdateEmailTemplateInput {
                realm_name,
                template_id,
                name: payload.name,
                structure: payload.structure,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateEmailTemplateResponse {
        data: template,
    }))
}

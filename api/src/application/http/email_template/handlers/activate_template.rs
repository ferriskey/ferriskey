use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    email_template::{
        entities::EmailTemplate,
        ports::{ActivateEmailTemplateInput, EmailTemplateService},
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
pub struct ActivateEmailTemplateResponse {
    pub data: EmailTemplate,
}

#[utoipa::path(
    patch,
    path = "/{template_id}/activate",
    tag = "email-template",
    summary = "Activate email template",
    description = "Activates an email template, deactivating any other active template of the same type in this realm.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("template_id" = Uuid, Path, description = "Email template ID"),
    ),
    responses(
        (status = 200, description = "Email template activated successfully", body = ActivateEmailTemplateResponse),
        (status = 404, description = "Email template not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn activate_template(
    Path((realm_name, template_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ActivateEmailTemplateResponse>, ApiError> {
    let template = state
        .service
        .activate_template(
            identity,
            ActivateEmailTemplateInput {
                realm_name,
                template_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ActivateEmailTemplateResponse {
        data: template,
    }))
}

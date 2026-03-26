use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    email_template::ports::{DeleteEmailTemplateInput, EmailTemplateService},
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteEmailTemplateResponse {
    message: String,
}

#[utoipa::path(
    delete,
    path = "/{template_id}",
    tag = "email-template",
    summary = "Delete email template",
    description = "Deletes an email template.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("template_id" = Uuid, Path, description = "Email template ID"),
    ),
    responses(
        (status = 200, description = "Email template deleted successfully", body = DeleteEmailTemplateResponse),
        (status = 404, description = "Email template not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_template(
    Path((realm_name, template_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteEmailTemplateResponse>, ApiError> {
    state
        .service
        .delete_template(
            identity,
            DeleteEmailTemplateInput {
                realm_name,
                template_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeleteEmailTemplateResponse {
        message: "Email template deleted successfully".to_string(),
    }))
}

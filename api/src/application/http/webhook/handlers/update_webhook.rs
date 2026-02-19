use crate::application::http::server::api_entities::api_error::{
    ApiError, ApiErrorResponse, ValidateJson,
};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::application::http::webhook::validators::UpdateWebhookValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::entities::webhook::Webhook;
use ferriskey_core::domain::webhook::ports::{UpdateWebhookInput, WebhookService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateWebhookResponse {
    pub data: Webhook,
}

#[utoipa::path(
    put,
    path = "",
    tag = "webhook",
    summary = "Update webhook",
    description = "Updates a webhook in the system related to the current realm.",
    responses(
        (status = 200, description = "Webhook updated successfully", body = UpdateWebhookResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]

pub async fn update_webhook(
    Path(realm_name): Path<String>,
    Path(webhook_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateWebhookValidator>,
) -> Result<Response<UpdateWebhookResponse>, ApiError> {
    let webhook = state
        .service
        .update_webhook(
            identity,
            UpdateWebhookInput {
                realm_name,
                webhook_id,
                name: payload.name,
                description: payload.description,
                endpoint: payload.endpoint,
                headers: payload.headers,
                subscribers: payload.subscribers,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateWebhookResponse { data: webhook }))
}

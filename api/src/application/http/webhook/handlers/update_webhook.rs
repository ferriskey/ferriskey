use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::application::http::webhook::validators::UpdateWebhookValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::webhook::use_cases::update_webhook_use_case::UpdateWebhookUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::entities::webhook::Webhook;
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
        (status = 200, body = Webhook)
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
        .use_case_bundle
        .update_webhook_use_case
        .execute(
            identity,
            UpdateWebhookUseCaseParams {
                realm_name,
                webhook_id,
                endpoint: payload.endpoint,
                subscribers: payload.subscribers,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UpdateWebhookResponse { data: webhook }))
}

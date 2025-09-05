use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::application::http::webhook::validators::CreateWebhookValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::entities::webhook::Webhook;
use ferriskey_core::domain::webhook::ports::{CreateWebhookInput, WebhookService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CreateWebhookResponse {
    pub data: Webhook,
}

#[utoipa::path(
    post,
    path = "",
    tag = "webhook",
    summary = "Create webhook",
    description = "Creates a new webhook in the system related to the current realm.",
    responses(
        (status = 200, body = CreateWebhookResponse)
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = CreateWebhookValidator
)]
pub async fn create_webhook(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateWebhookValidator>,
) -> Result<Response<CreateWebhookResponse>, ApiError> {
    let webhook = state
        .service
        .create_webhook(
            identity,
            CreateWebhookInput {
                realm_name,
                name: payload.name,
                description: payload.description,
                endpoint: payload.endpoint,
                subscribers: payload.subscribers,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(CreateWebhookResponse { data: webhook }))
}

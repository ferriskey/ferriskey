use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::application::http::webhook::validators::CreateWebhookValidator;
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::webhook::use_cases::create_webhook_use_case::CreateWebhookUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::entities::webhook::Webhook;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/webhooks")]
pub struct CreateWebhookRoute {
    realm_name: String,
}

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
        (status = 200, body = Webhook)
    ),
)]

pub async fn create_webhook(
    CreateWebhookRoute { realm_name }: CreateWebhookRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateWebhookValidator>,
) -> Result<Response<CreateWebhookResponse>, ApiError> {
    let webhook = state
        .use_case_bundle
        .create_webhook_use_case
        .execute(
            identity,
            CreateWebhookUseCaseParams {
                realm_name,
                endpoint: payload.endpoint,
                subscribers: payload.subscribers,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(CreateWebhookResponse { data: webhook }))
}

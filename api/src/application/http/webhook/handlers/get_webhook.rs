use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::webhook::use_cases::get_webhook_use_case::GetWebhookUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::entities::webhook::Webhook;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{webhook_id}",
    tag = "webhook",
    summary = "Get webhook",
    description = "Retrieves one webhook in the system related to the current realm.",
    params(
        ("webhook_id" = Uuid, Path, description = "Webhook ID"),
    ),
    responses(
        (status = 200, body = Webhook)
    ),
)]

pub async fn get_webhook(
    Path(realm_name): Path<String>,
    Path(webhook_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Option<Webhook>>, ApiError> {
    let webhook = state
        .use_case_bundle
        .get_webhook_use_case
        .execute(
            identity,
            GetWebhookUseCaseParams {
                realm_name,
                webhook_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(webhook))
}

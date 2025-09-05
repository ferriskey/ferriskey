use crate::application::http::server::api_entities::{api_error::ApiError, response::Response};
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::ports::{DeleteWebhookInput, WebhookService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteWebhookResponse {
    message: String,
    realm_name: String,
}

#[utoipa::path(
    delete,
    path = "/{webhook_id}",
    tag = "webhook",
    summary = "Delete webhook",
    description = "Deletes a webhook in the system related to the current realm.",
    responses(
        (status = 200, body = DeleteWebhookResponse)
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("webhook_id" = Uuid, Path, description = "Webhook ID"),
    )
)]
pub async fn delete_webhook(
    Path((realm_name, webhook_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteWebhookResponse>, ApiError> {
    state
        .service
        .delete_webhook(
            identity,
            DeleteWebhookInput {
                realm_name: realm_name.clone(),
                webhook_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeleteWebhookResponse {
        message: "".to_string(),
        realm_name,
    }))
}

use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::entities::webhook_delivery::WebhookDeliveryId;
use ferriskey_core::domain::webhook::ports::{RetryWebhookDeliveryInput, WebhookService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct RetryDeliveryResponse {
    pub delivery_id: Uuid,
    pub requeued: bool,
}

#[utoipa::path(
    post,
    path = "/{webhook_id}/deliveries/{delivery_id}/retry",
    tag = "webhook",
    summary = "Replay a webhook delivery",
    description = "Puts a terminal delivery back in the queue so the worker attempts it again.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("webhook_id" = Uuid, Path, description = "Webhook ID"),
        ("delivery_id" = Uuid, Path, description = "Delivery ID"),
    ),
    responses(
        (status = 202, description = "Delivery queued for another attempt", body = RetryDeliveryResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Delivery not found", body = ApiErrorResponse),
        (status = 409, description = "Delivery is still in flight", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn retry_delivery(
    Path((realm_name, webhook_id, delivery_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<RetryDeliveryResponse>, ApiError> {
    state
        .service
        .retry_webhook_delivery(
            identity,
            RetryWebhookDeliveryInput {
                realm_name,
                webhook_id,
                delivery_id: WebhookDeliveryId::from_uuid(delivery_id),
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Accepted(RetryDeliveryResponse {
        delivery_id,
        requeued: true,
    }))
}

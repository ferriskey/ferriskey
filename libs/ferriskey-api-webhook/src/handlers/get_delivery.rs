use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::entities::webhook_delivery::{
    WebhookDelivery, WebhookDeliveryId,
};
use ferriskey_core::domain::webhook::ports::{GetWebhookDeliveryInput, WebhookService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::fetch_deliveries::DeliverySummary;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct DeliveryDetail {
    #[serde(flatten)]
    pub summary: DeliverySummary,
    pub payload: serde_json::Value,
}

impl From<WebhookDelivery> for DeliveryDetail {
    fn from(value: WebhookDelivery) -> Self {
        Self {
            payload: value.payload.clone(),
            summary: DeliverySummary::from(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct GetDeliveryResponse {
    pub data: DeliveryDetail,
}

#[utoipa::path(
    get,
    path = "/{webhook_id}/deliveries/{delivery_id}",
    tag = "webhook",
    summary = "Get a webhook delivery",
    description = "Retrieves one recorded delivery, including the payload that was sent.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("webhook_id" = Uuid, Path, description = "Webhook ID"),
        ("delivery_id" = Uuid, Path, description = "Delivery ID"),
    ),
    responses(
        (status = 200, description = "Delivery retrieved successfully", body = GetDeliveryResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Delivery not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_delivery(
    Path((realm_name, webhook_id, delivery_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetDeliveryResponse>, ApiError> {
    let delivery = state
        .service
        .get_webhook_delivery(
            identity,
            GetWebhookDeliveryInput {
                realm_name,
                webhook_id,
                delivery_id: WebhookDeliveryId::from_uuid(delivery_id),
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetDeliveryResponse {
        data: DeliveryDetail::from(delivery),
    }))
}

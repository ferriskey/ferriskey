use axum::{
    Extension,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::entities::webhook_delivery::{
    DeliveryFilter, DeliveryStatus, WebhookDelivery,
};
use ferriskey_core::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use ferriskey_core::domain::webhook::ports::{GetWebhookDeliveriesInput, WebhookService};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct DeliverySummary {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event: WebhookTrigger,
    pub resource_id: Uuid,
    pub status: String,
    pub attempt_count: u32,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_status_code: Option<u16>,
    pub last_error_code: Option<String>,
    pub last_error_detail: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WebhookDelivery> for DeliverySummary {
    fn from(value: WebhookDelivery) -> Self {
        Self {
            id: value.id.as_uuid(),
            webhook_id: value.webhook_id,
            event: value.event,
            resource_id: value.resource_id,
            status: value.status.as_str().to_string(),
            attempt_count: value.attempt_count,
            next_attempt_at: value.next_attempt_at,
            last_attempt_at: value.last_attempt_at,
            last_status_code: value.last_status_code,
            last_error_code: value.last_error_code.map(|code| code.as_code()),
            last_error_detail: value.last_error_detail,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct FetchDeliveriesResponse {
    pub data: Vec<DeliverySummary>,
    pub total: u64,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct FetchDeliveriesQuery {
    pub status: Option<String>,
    pub event: Option<WebhookTrigger>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl TryFrom<FetchDeliveriesQuery> for DeliveryFilter {
    type Error = ApiError;

    fn try_from(query: FetchDeliveriesQuery) -> Result<Self, Self::Error> {
        let status = match query.status.as_deref() {
            Some(raw) => Some(
                DeliveryStatus::parse(raw)
                    .ok_or_else(|| ApiError::BadRequest(format!("unknown status: {raw}").into()))?,
            ),
            None => None,
        };

        DeliveryFilter::new(status, query.event, query.limit, query.offset)
            .map_err(|error| ApiError::BadRequest(error.to_string().into()))
    }
}

#[utoipa::path(
    get,
    path = "/{webhook_id}/deliveries",
    tag = "webhook",
    summary = "List webhook deliveries",
    description = "Lists the recorded delivery attempts for one webhook, newest first.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("webhook_id" = Uuid, Path, description = "Webhook ID"),
        FetchDeliveriesQuery,
    ),
    responses(
        (status = 200, description = "Deliveries retrieved successfully", body = FetchDeliveriesResponse),
        (status = 400, description = "Invalid filter or page size", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn fetch_deliveries(
    Path((realm_name, webhook_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<FetchDeliveriesQuery>,
) -> Result<Response<FetchDeliveriesResponse>, ApiError> {
    let filter = DeliveryFilter::try_from(query)?;

    let page = state
        .service
        .get_webhook_deliveries(
            identity,
            GetWebhookDeliveriesInput {
                realm_name,
                webhook_id,
                filter,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(FetchDeliveriesResponse {
        data: page.items.into_iter().map(DeliverySummary::from).collect(),
        total: page.total,
    }))
}

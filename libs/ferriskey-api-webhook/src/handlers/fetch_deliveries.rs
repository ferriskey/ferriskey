use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    list_query::{ListQuery, PaginationParams},
    paginated::Paginated,
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::common::pagination::DateRange;
use ferriskey_core::domain::webhook::entities::webhook_delivery::{
    DeliveryStatus, WebhookDelivery, WebhookDeliveryFilter, WebhookDeliverySortField,
};
use ferriskey_core::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use ferriskey_core::domain::webhook::ports::WebhookService;
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

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct WebhookDeliveryListParams {
    pub search: Option<String>,
    #[param(inline)]
    pub event: Option<WebhookTrigger>,
    #[param(inline)]
    pub status: Option<DeliveryStatus>,
    pub resource_id: Option<Uuid>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
}

impl From<WebhookDeliveryListParams> for WebhookDeliveryFilter {
    fn from(params: WebhookDeliveryListParams) -> Self {
        Self {
            search: params.search,
            event: params.event,
            status: params.status,
            resource_id: params.resource_id,
            created: DateRange::new(params.created_from, params.created_to),
        }
    }
}

#[utoipa::path(
    get,
    path = "/{webhook_id}/deliveries",
    tag = "webhook",
    summary = "List webhook deliveries",
    description = "Returns one page of the recorded delivery attempts of one webhook of the realm. Payloads are not included; read a single delivery to get its payload. search matches case-insensitively a delivery whose resource_id, written as a lowercase hyphenated UUID, contains the value. event matches one webhook trigger exactly; status matches exactly one of pending, delivering, succeeded, failed; resource_id matches the id of the resource the event is about. created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. Deliveries never attempted sort last on last_attempt_at in ascending order and first in descending order.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("webhook_id" = Uuid, Path, description = "Webhook ID"),
        PaginationParams,
        WebhookDeliveryListParams,
        ("order_by" = inline(Option<WebhookDeliverySortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of deliveries", body = Paginated<DeliverySummary>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm or webhook not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn fetch_deliveries(
    Path((realm_name, webhook_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<WebhookDeliveryListParams, WebhookDeliverySortField>,
) -> Result<Response<Paginated<DeliverySummary>>, ApiError> {
    let page = state
        .service
        .list_webhook_deliveries(
            identity,
            realm_name,
            webhook_id,
            request.map_filter(WebhookDeliveryFilter::from),
        )
        .await?;

    Ok(Response::OK(Paginated::from(
        page.map(DeliverySummary::from),
    )))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use ferriskey_api_core::api_entities::list_query::parse_list_query;
    use ferriskey_core::domain::common::pagination::SortOrder;

    use super::*;

    #[test]
    fn every_filter_and_sort_field_is_read() {
        let resource = Uuid::from_u128(7);
        let request = parse_list_query::<WebhookDeliveryListParams, WebhookDeliverySortField>(
            &format!(
                "order_by=last_attempt_at&order=asc&search=ab-cd&event=user.created&status=failed&resource_id={resource}&created_from=2026-01-01T00:00:00Z&created_to=2026-02-01T00:00:00%2B02:00"
            ),
        )
        .expect("valid query");

        assert_eq!(request.sort.field, WebhookDeliverySortField::LastAttemptAt);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            WebhookDeliveryFilter::from(request.filter),
            WebhookDeliveryFilter {
                search: Some("ab-cd".to_string()),
                event: Some(WebhookTrigger::UserCreated),
                status: Some(DeliveryStatus::Failed),
                resource_id: Some(resource),
                created: DateRange::new(
                    Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                    Utc.with_ymd_and_hms(2026, 1, 31, 22, 0, 0).single(),
                ),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in [
            "status",
            "attempt_count",
            "last_attempt_at",
            "created_at",
            "updated_at",
        ] {
            assert!(
                parse_list_query::<WebhookDeliveryListParams, WebhookDeliverySortField>(&format!(
                    "order_by={value}"
                ))
                .is_ok(),
                "{value}"
            );
        }
    }

    #[test]
    fn unknown_filters_and_columns_are_refused() {
        for query in [
            "offset=10",
            "payload=x",
            "realm_id=x",
            "webhook_id=x",
            "order_by=payload",
            "order_by=next_attempt_at",
            "order_by=event",
            "status=exploded",
            "status=FAILED",
            "event=user.exploded",
            "event=%25",
            "resource_id=nope",
            "search=a&search=b",
            "created_from=2026-10-05",
            "created_to=2026-10-05",
        ] {
            assert!(
                parse_list_query::<WebhookDeliveryListParams, WebhookDeliverySortField>(query)
                    .is_err(),
                "{query}"
            );
        }
    }
}

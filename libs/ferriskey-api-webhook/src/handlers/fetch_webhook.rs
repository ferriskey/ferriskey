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
use ferriskey_core::domain::webhook::entities::webhook::{
    Webhook, WebhookFilter, WebhookSortField,
};
use ferriskey_core::domain::webhook::ports::WebhookService;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct WebhookListParams {
    pub search: Option<String>,
    pub name: Option<String>,
    pub endpoint: Option<String>,
    pub triggered: Option<bool>,
    pub has_subscribers: Option<bool>,
    pub secure_endpoint: Option<bool>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
}

impl From<WebhookListParams> for WebhookFilter {
    fn from(params: WebhookListParams) -> Self {
        Self {
            search: params.search,
            name: params.name,
            endpoint: params.endpoint,
            triggered: params.triggered,
            has_subscribers: params.has_subscribers,
            secure_endpoint: params.secure_endpoint,
            created: DateRange::new(params.created_from, params.created_to),
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    tag = "webhook",
    summary = "List the webhooks of a realm",
    description = "Returns one page of the realm's webhooks, each with its subscribers. Stored secrets and header values are never returned. search matches case-insensitively a webhook whose name or endpoint contains the value; a webhook without a name only matches by its endpoint. Text filters (name, endpoint) match case-insensitively anywhere in the value; triggered keeps webhooks that fired at least once (true) or never (false); has_subscribers keeps webhooks with at least one subscribed trigger (true) or none (false); secure_endpoint keeps endpoints starting with `https://` (true) or not (false). created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. Webhooks without a name or never triggered sort last in ascending order and first in descending order.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        PaginationParams,
        WebhookListParams,
        ("order_by" = inline(Option<WebhookSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of webhooks", body = Paginated<Webhook>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn fetch_webhooks(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<WebhookListParams, WebhookSortField>,
) -> Result<Response<Paginated<Webhook>>, ApiError> {
    let page = state
        .service
        .list_webhooks(
            identity,
            realm_name,
            request.map_filter(WebhookFilter::from),
        )
        .await?;

    Ok(Response::OK(Paginated::from(page)))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use ferriskey_api_core::api_entities::list_query::parse_list_query;
    use ferriskey_core::domain::common::pagination::SortOrder;

    use super::*;

    #[test]
    fn every_filter_and_sort_field_is_read() {
        let request = parse_list_query::<WebhookListParams, WebhookSortField>(
            "order_by=triggered_at&order=asc&search=e0&created_from=2026-01-01T00:00:00Z&created_to=2026-02-01T00:00:00%2B02:00&name=hook&endpoint=example&triggered=true&has_subscribers=false&secure_endpoint=false",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, WebhookSortField::TriggeredAt);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            WebhookFilter::from(request.filter),
            WebhookFilter {
                search: Some("e0".to_string()),
                name: Some("hook".to_string()),
                endpoint: Some("example".to_string()),
                triggered: Some(true),
                has_subscribers: Some(false),
                secure_endpoint: Some(false),
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
            "name",
            "endpoint",
            "triggered_at",
            "created_at",
            "updated_at",
        ] {
            assert!(
                parse_list_query::<WebhookListParams, WebhookSortField>(&format!(
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
            "realm_id=x",
            "secret=x",
            "headers=x",
            "order_by=secret",
            "order_by=last_delivery_status",
            "triggered=maybe",
            "has_subscribers=maybe",
            "secure_endpoint=maybe",
            "last_delivery_status=failed",
            "search=a&search=b",
            "created_from=2026-10-05",
            "created_to=2026-10-05",
        ] {
            assert!(
                parse_list_query::<WebhookListParams, WebhookSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

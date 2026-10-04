use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    list_query::{ListQuery, PaginationParams},
    paginated::Paginated,
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
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
    pub name: Option<String>,
    pub endpoint: Option<String>,
    #[param(example = "failed")]
    pub last_delivery_status: Option<String>,
    pub triggered: Option<bool>,
    pub has_subscribers: Option<bool>,
    pub secure_endpoint: Option<bool>,
}

impl From<WebhookListParams> for WebhookFilter {
    fn from(params: WebhookListParams) -> Self {
        Self {
            name: params.name,
            endpoint: params.endpoint,
            last_delivery_status: params.last_delivery_status,
            triggered: params.triggered,
            has_subscribers: params.has_subscribers,
            secure_endpoint: params.secure_endpoint,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    tag = "webhook",
    summary = "List the webhooks of a realm",
    description = "Returns one page of the realm's webhooks, each with its subscribers. Stored secrets and header values are never returned. Text filters (name, endpoint) match case-insensitively anywhere in the value; last_delivery_status matches exactly; triggered keeps webhooks that fired at least once (true) or never (false); has_subscribers keeps webhooks with at least one subscribed trigger (true) or none (false); secure_endpoint keeps endpoints starting with `https://` (true) or not (false). Filters combine with AND. Webhooks without a name or never triggered sort last in ascending order and first in descending order.",
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
    use ferriskey_api_core::api_entities::list_query::parse_list_query;
    use ferriskey_core::domain::common::pagination::SortOrder;

    use super::*;

    #[test]
    fn every_filter_and_sort_field_is_read() {
        let request = parse_list_query::<WebhookListParams, WebhookSortField>(
            "order_by=triggered_at&order=asc&name=hook&endpoint=example&last_delivery_status=failed&triggered=true&has_subscribers=false&secure_endpoint=false",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, WebhookSortField::TriggeredAt);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            WebhookFilter::from(request.filter),
            WebhookFilter {
                name: Some("hook".to_string()),
                endpoint: Some("example".to_string()),
                last_delivery_status: Some("failed".to_string()),
                triggered: Some(true),
                has_subscribers: Some(false),
                secure_endpoint: Some(false),
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
        ] {
            assert!(
                parse_list_query::<WebhookListParams, WebhookSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

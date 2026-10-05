use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    common::pagination::PageRequest,
    seawatch::{
        EventStatus, SecurityEvent, SecurityEventFilter, SecurityEventSortField, SecurityEventType,
        ports::SecurityEventService,
    },
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorBody, ApiErrorResponse},
        list_query::{ListQuery, PaginationParams},
        paginated::Paginated,
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct SecurityEventListParams {
    #[param(example = "10.0.")]
    pub search: Option<String>,
    pub actor_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    #[param(example = "login_failure,session_revoked")]
    pub event_types: Option<String>,
    #[param(inline)]
    pub status: Option<EventStatus>,
    #[param(example = "client")]
    pub target_type: Option<String>,
    #[param(example = "2026-01-01T00:00:00Z")]
    pub from_timestamp: Option<DateTime<Utc>>,
    #[param(example = "2026-01-31T23:59:59Z")]
    pub to_timestamp: Option<DateTime<Utc>>,
    #[param(example = "10.0.")]
    pub ip_address: Option<String>,
}

fn parse_event_types(raw: &str) -> Result<Vec<SecurityEventType>, ApiError> {
    raw.split(',')
        .map(|part| match SecurityEventType::parse(part.trim()) {
            SecurityEventType::Unknown => Err(ApiError::BadRequest(ApiErrorBody::new(
                format!("Invalid query parameter `event_types`: unknown event type `{part}`"),
                "invalid_query",
            ))),
            known => Ok(known),
        })
        .collect()
}

impl TryFrom<SecurityEventListParams> for SecurityEventFilter {
    type Error = ApiError;

    fn try_from(params: SecurityEventListParams) -> Result<Self, Self::Error> {
        Ok(Self {
            search: params.search,
            client_id: params.client_id,
            actor_id: params.actor_id,
            event_types: params
                .event_types
                .as_deref()
                .map(parse_event_types)
                .transpose()?,
            status: params.status,
            target_type: params.target_type,
            from_timestamp: params.from_timestamp,
            to_timestamp: params.to_timestamp,
            ip_address: params.ip_address,
        })
    }
}

#[utoipa::path(
    get,
    summary = "Get Security Events",
    description = "Returns one page of the realm's security events. search matches case-insensitively anywhere in the ip_address and never matches an event without an address. ip_address matches case-insensitively anywhere in the value and never matches an event without an address. event_types takes a comma-separated list of event types and keeps the events of any of them. status, target_type, actor_id and client_id (the event's target id) match exactly. from_timestamp and to_timestamp bound timestamp, both inclusive. Filters combine with AND.",
    path = "/seawatch/v1/security-events",
    tag = "seawatch",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        PaginationParams,
        SecurityEventListParams,
        ("order_by" = inline(Option<SecurityEventSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of security events", body = Paginated<SecurityEvent>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_security_events(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<SecurityEventListParams, SecurityEventSortField>,
) -> Result<Response<Paginated<SecurityEvent>>, ApiError> {
    let request = PageRequest {
        filter: SecurityEventFilter::try_from(request.filter)?,
        page: request.page,
        limit: request.limit,
        sort: request.sort,
    };
    let page = state
        .service
        .list_events(identity, realm_name, request)
        .await?;

    Ok(Response::OK(Paginated::from(page)))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use ferriskey_api_core::api_entities::list_query::parse_list_query;
    use ferriskey_core::domain::common::pagination::SortOrder;

    use super::*;

    fn parse(
        query: &str,
    ) -> Result<PageRequest<SecurityEventFilter, SecurityEventSortField>, ApiError> {
        let request = parse_list_query::<SecurityEventListParams, SecurityEventSortField>(query)?;
        Ok(PageRequest {
            filter: SecurityEventFilter::try_from(request.filter)?,
            page: request.page,
            limit: request.limit,
            sort: request.sort,
        })
    }

    fn message(error: ApiError) -> String {
        match error {
            ApiError::BadRequest(body) => body.message.into_owned(),
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }

    #[test]
    fn every_filter_and_sort_field_is_read() {
        let actor = Uuid::new_v4();
        let client = Uuid::new_v4();
        let request = parse(&format!(
            "order_by=event_type&order=asc&search=0.1&actor_id={actor}&client_id={client}&event_types=login_failure,session_revoked&status=failure&target_type=client&from_timestamp=2026-01-01T00:00:00Z&to_timestamp=2026-01-02T00:00:00Z&ip_address=10.0"
        ))
        .expect("valid query");

        assert_eq!(request.sort.field, SecurityEventSortField::EventType);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            request.filter,
            SecurityEventFilter {
                search: Some("0.1".to_string()),
                client_id: Some(client),
                actor_id: Some(actor),
                event_types: Some(vec![
                    SecurityEventType::LoginFailure,
                    SecurityEventType::SessionRevoked,
                ]),
                status: Some(EventStatus::Failure),
                target_type: Some("client".to_string()),
                from_timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                to_timestamp: Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).single(),
                ip_address: Some("10.0".to_string()),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["event_type", "status", "timestamp", "created_at"] {
            assert!(parse(&format!("order_by={value}")).is_ok(), "{value}");
        }
    }

    #[test]
    fn unknown_filters_values_and_columns_are_refused() {
        for (query, param) in [
            ("offset=10", "offset"),
            ("limit=1000", "limit"),
            ("user_agent=firefox", "user_agent"),
            ("search=a&search=b", "search"),
            ("created_from=2026-01-01T00:00:00Z", "created_from"),
            ("created_to=2026-01-01T00:00:00Z", "created_to"),
            ("order_by=ip_address", "order_by"),
            ("order_by=actor_id", "order_by"),
            ("status=done", "status"),
            ("actor_id=nope", "actor_id"),
            ("from_timestamp=yesterday", "from_timestamp"),
            ("event_types=nope", "event_types"),
            ("event_types=login_failure,", "event_types"),
            ("event_types=unknown", "event_types"),
        ] {
            let text = message(parse(query).expect_err(query));
            assert!(text.contains(param), "{query}: {text}");
        }
    }
}

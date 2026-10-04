use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    compass::{
        entities::{CompassFlow, FlowStatus},
        ports::CompassService,
        value_objects::{FlowFilter, FlowSortField},
    },
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        list_query::{ListQuery, PaginationParams},
        paginated::Paginated,
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct FlowListParams {
    #[param(example = "web-app")]
    pub client_id: Option<String>,
    pub user_id: Option<Uuid>,
    #[param(example = "authorization_code")]
    pub grant_type: Option<String>,
    #[param(inline)]
    pub status: Option<FlowStatus>,
    #[param(example = "192.168.")]
    pub ip_address: Option<String>,
    pub identified: Option<bool>,
    pub completed: Option<bool>,
    #[param(example = "2026-01-01T00:00:00Z")]
    pub from: Option<DateTime<Utc>>,
    #[param(example = "2026-01-31T23:59:59Z")]
    pub to: Option<DateTime<Utc>>,
}

impl From<FlowListParams> for FlowFilter {
    fn from(params: FlowListParams) -> Self {
        Self {
            client_id: params.client_id,
            user_id: params.user_id,
            grant_type: params.grant_type,
            status: params.status,
            ip_address: params.ip_address,
            identified: params.identified,
            completed: params.completed,
            from_timestamp: params.from,
            to_timestamp: params.to,
        }
    }
}

#[utoipa::path(
    get,
    summary = "Get Compass Flows",
    description = "Returns one page of the realm's authentication flows, with their steps. ip_address matches case-insensitively anywhere in the value and never matches a flow without an address. client_id (the OAuth client_id string), user_id, grant_type and status match exactly. identified=true keeps flows tied to a user, identified=false the anonymous ones; completed=true keeps flows with a completion date, completed=false the unfinished ones. from and to bound started_at, both inclusive. Filters combine with AND. Sorting on duration_ms puts unfinished flows last in ascending order and first in descending order.",
    path = "/compass/v1/flows",
    tag = "compass",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        PaginationParams,
        FlowListParams,
        ("order_by" = inline(Option<FlowSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of flows", body = Paginated<CompassFlow>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_flows(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<FlowListParams, FlowSortField>,
) -> Result<Response<Paginated<CompassFlow>>, ApiError> {
    let page = state
        .service
        .list_flows(identity, realm_name, request.map_filter(Into::into))
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
        let user_id = Uuid::new_v4();
        let request = parse_list_query::<FlowListParams, FlowSortField>(&format!(
            "order_by=duration_ms&order=asc&client_id=web-app&user_id={user_id}&grant_type=password&status=failure&ip_address=10.0&identified=false&completed=true&from=2026-01-01T00:00:00Z&to=2026-01-02T00:00:00Z"
        ))
        .expect("valid query");

        assert_eq!(request.sort.field, FlowSortField::DurationMs);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            FlowFilter::from(request.filter),
            FlowFilter {
                client_id: Some("web-app".to_string()),
                user_id: Some(user_id),
                grant_type: Some("password".to_string()),
                status: Some(FlowStatus::Failure),
                ip_address: Some("10.0".to_string()),
                identified: Some(false),
                completed: Some(true),
                from_timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                to_timestamp: Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).single(),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["status", "started_at", "duration_ms", "created_at"] {
            assert!(
                parse_list_query::<FlowListParams, FlowSortField>(&format!("order_by={value}"))
                    .is_ok(),
                "{value}"
            );
        }
    }

    #[test]
    fn unknown_filters_values_and_columns_are_refused() {
        for query in [
            "offset=10",
            "search=x",
            "user_agent=firefox",
            "order_by=ip_address",
            "order_by=grant_type",
            "status=done",
            "user_id=nope",
            "from=yesterday",
            "identified=maybe",
            "completed=1",
        ] {
            assert!(
                parse_list_query::<FlowListParams, FlowSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

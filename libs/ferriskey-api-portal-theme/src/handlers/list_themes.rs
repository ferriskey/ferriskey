use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    common::pagination::DateRange,
    portal_theme::{
        entities::{PortalTheme, PortalThemeFilter, PortalThemeSortField},
        ports::PortalThemeService,
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
pub struct PortalThemeListParams {
    pub search: Option<String>,
    pub name: Option<String>,
    pub layout_id: Option<Uuid>,
    pub activatable: Option<bool>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
}

impl From<PortalThemeListParams> for PortalThemeFilter {
    fn from(params: PortalThemeListParams) -> Self {
        Self {
            search: params.search,
            name: params.name,
            layout_id: params.layout_id,
            activatable: params.activatable,
            created: DateRange::new(params.created_from, params.created_to),
        }
    }
}

#[utoipa::path(
    get,
    path = "/portal/themes",
    tag = "portal-theme",
    summary = "List portal themes",
    description = "Returns one page of the realm's portal themes, each with its design tokens and page trees. search and the name filter match case-insensitively anywhere in the name; the name filter matches case-insensitively anywhere in the value; layout_id matches exactly; activatable=true keeps themes whose every page holds the blocks listed by page-requirements (the check activation runs), activatable=false keeps the others; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. Requires manage_realm permission.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        PaginationParams,
        PortalThemeListParams,
        ("order_by" = inline(Option<PortalThemeSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of portal themes", body = Paginated<PortalTheme>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_themes(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<PortalThemeListParams, PortalThemeSortField>,
) -> Result<Response<Paginated<PortalTheme>>, ApiError> {
    let page = state
        .service
        .list_themes(
            identity,
            realm_name,
            request.map_filter(PortalThemeFilter::from),
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
        let request = parse_list_query::<PortalThemeListParams, PortalThemeSortField>(
            "order_by=updated_at&order=asc&search=br&name=brand&layout_id=00000000-0000-0000-0000-000000000001&activatable=false&created_from=2026-01-01T00:00:00Z&created_to=2026-02-01T00:00:00%2B02:00",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, PortalThemeSortField::UpdatedAt);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            PortalThemeFilter::from(request.filter),
            PortalThemeFilter {
                search: Some("br".to_string()),
                name: Some("brand".to_string()),
                layout_id: Some(Uuid::from_u128(1)),
                activatable: Some(false),
                created: DateRange::new(
                    Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                    Utc.with_ymd_and_hms(2026, 1, 31, 22, 0, 0).single(),
                ),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["name", "created_at", "updated_at"] {
            assert!(
                parse_list_query::<PortalThemeListParams, PortalThemeSortField>(&format!(
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
            "config=x",
            "order_by=layout_id",
            "order_by=config",
            "layout_id=nope",
            "activatable=maybe",
            "search=a&search=b",
            "created_from=2026-10-05",
            "created_to=2026-10-05",
        ] {
            assert!(
                parse_list_query::<PortalThemeListParams, PortalThemeSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

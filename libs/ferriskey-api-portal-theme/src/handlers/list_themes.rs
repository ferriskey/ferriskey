use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
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
    pub name: Option<String>,
    pub layout_id: Option<Uuid>,
    pub activatable: Option<bool>,
}

impl From<PortalThemeListParams> for PortalThemeFilter {
    fn from(params: PortalThemeListParams) -> Self {
        Self {
            name: params.name,
            layout_id: params.layout_id,
            activatable: params.activatable,
        }
    }
}

#[utoipa::path(
    get,
    path = "/portal/themes",
    tag = "portal-theme",
    summary = "List portal themes",
    description = "Returns one page of the realm's portal themes, each with its design tokens and page trees. The name filter matches case-insensitively anywhere in the value; layout_id matches exactly; activatable=true keeps themes whose every page holds the blocks listed by page-requirements (the check activation runs), activatable=false keeps the others. Filters combine with AND. Requires manage_realm permission.",
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
    use ferriskey_api_core::api_entities::list_query::parse_list_query;
    use ferriskey_core::domain::common::pagination::SortOrder;

    use super::*;

    #[test]
    fn every_filter_and_sort_field_is_read() {
        let request = parse_list_query::<PortalThemeListParams, PortalThemeSortField>(
            "order_by=updated_at&order=asc&name=brand&layout_id=00000000-0000-0000-0000-000000000001&activatable=false",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, PortalThemeSortField::UpdatedAt);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            PortalThemeFilter::from(request.filter),
            PortalThemeFilter {
                name: Some("brand".to_string()),
                layout_id: Some(Uuid::from_u128(1)),
                activatable: Some(false),
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
        ] {
            assert!(
                parse_list_query::<PortalThemeListParams, PortalThemeSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

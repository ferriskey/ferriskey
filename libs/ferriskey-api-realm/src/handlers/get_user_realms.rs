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
use ferriskey_core::domain::realm::entities::{Realm, RealmFilter, RealmSortField};
use ferriskey_core::domain::realm::ports::RealmService;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct RealmListParams {
    pub name: Option<String>,
    pub display_name: Option<String>,
}

impl From<RealmListParams> for RealmFilter {
    fn from(params: RealmListParams) -> Self {
        Self {
            name: params.name,
            display_name: params.display_name,
        }
    }
}

#[utoipa::path(
    get,
    summary = "Get user realms",
    description = "Returns one page of the realms the caller may access. Text filters (name, display_name) match case-insensitively anywhere in the value and combine with AND.",
    path = "/{realm_name}/users/@me/realms",
    tag = "realm",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        PaginationParams,
        RealmListParams,
        ("order_by" = inline(Option<RealmSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    security(
        ("Authorization" = ["Bearer"]),
    ),
    responses(
        (status = 200, description = "One page of the caller's realms", body = Paginated<Realm>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_user_realms(
    Path(_): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<RealmListParams, RealmSortField>,
) -> Result<Response<Paginated<Realm>>, ApiError> {
    let page = state
        .service
        .list_user_realms(identity, request.map_filter(RealmFilter::from))
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
        let request = parse_list_query::<RealmListParams, RealmSortField>(
            "order_by=name&order=asc&name=ma&display_name=Main",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, RealmSortField::Name);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            RealmFilter::from(request.filter),
            RealmFilter {
                name: Some("ma".to_string()),
                display_name: Some("Main".to_string()),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["name", "created_at", "updated_at"] {
            assert!(
                parse_list_query::<RealmListParams, RealmSortField>(&format!("order_by={value}"))
                    .is_ok(),
                "{value}"
            );
        }
    }

    #[test]
    fn unknown_filters_and_columns_are_refused() {
        for query in ["id=x", "order_by=display_name", "order_by=id"] {
            assert!(
                parse_list_query::<RealmListParams, RealmSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

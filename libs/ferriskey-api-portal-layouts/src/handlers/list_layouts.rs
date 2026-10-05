use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    common::pagination::PageRequest,
    portal_layouts::{
        entities::{PortalLayoutFilter, PortalLayoutListItem, PortalLayoutSortField},
        ports::PortalLayoutsService,
    },
};
use serde::Deserialize;
use utoipa::IntoParams;

use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        list_query::{ListQuery, PaginationParams, parse_id_list},
        paginated::Paginated,
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct PortalLayoutListParams {
    pub name: Option<String>,
    pub is_default: Option<bool>,
    pub in_use: Option<bool>,
    #[param(example = "0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e6f,0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e70")]
    pub ids: Option<String>,
}

impl TryFrom<PortalLayoutListParams> for PortalLayoutFilter {
    type Error = ApiError;

    fn try_from(params: PortalLayoutListParams) -> Result<Self, Self::Error> {
        Ok(Self {
            name: params.name,
            is_default: params.is_default,
            in_use: params.in_use,
            ids: params
                .ids
                .as_deref()
                .map(|raw| parse_id_list("ids", raw))
                .transpose()?,
        })
    }
}

#[utoipa::path(
    get,
    path = "",
    tag = "portal-layouts",
    summary = "List portal layouts",
    description = "Returns one page of the realm's portal layouts, each with its tree and theme_count, the number of the realm's themes built on it. The name filter matches case-insensitively anywhere in the value; is_default matches exactly; in_use=true keeps layouts at least one theme of the realm uses, in_use=false keeps the others; ids takes a comma-separated list of at most 100 layout ids. Filters combine with AND. Requires manage_realm permission.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        PaginationParams,
        PortalLayoutListParams,
        ("order_by" = inline(Option<PortalLayoutSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of portal layouts", body = Paginated<PortalLayoutListItem>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_layouts(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<PortalLayoutListParams, PortalLayoutSortField>,
) -> Result<Response<Paginated<PortalLayoutListItem>>, ApiError> {
    let request = PageRequest {
        filter: PortalLayoutFilter::try_from(request.filter)?,
        page: request.page,
        limit: request.limit,
        sort: request.sort,
    };
    let page = state
        .service
        .list_layouts(identity, realm_name, request)
        .await?;

    Ok(Response::OK(Paginated::from(page)))
}

#[cfg(test)]
mod tests {
    use ferriskey_api_core::api_entities::list_query::parse_list_query;
    use ferriskey_core::domain::common::pagination::SortOrder;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn every_filter_and_sort_field_is_read() {
        let request = parse_list_query::<PortalLayoutListParams, PortalLayoutSortField>(
            "order_by=name&order=asc&name=shell&is_default=false&in_use=true&ids=00000000-0000-0000-0000-000000000001,00000000-0000-0000-0000-000000000002",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, PortalLayoutSortField::Name);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            PortalLayoutFilter::try_from(request.filter).expect("valid filter"),
            PortalLayoutFilter {
                name: Some("shell".to_string()),
                is_default: Some(false),
                in_use: Some(true),
                ids: Some(vec![Uuid::from_u128(1), Uuid::from_u128(2)]),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["name", "created_at", "updated_at"] {
            assert!(
                parse_list_query::<PortalLayoutListParams, PortalLayoutSortField>(&format!(
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
            "tree=x",
            "theme_count=1",
            "order_by=tree",
            "order_by=theme_count",
            "is_default=maybe",
            "in_use=maybe",
        ] {
            assert!(
                parse_list_query::<PortalLayoutListParams, PortalLayoutSortField>(query).is_err(),
                "{query}"
            );
        }
    }

    #[test]
    fn invalid_ids_name_the_parameter() {
        let hundred_and_one = (0..101)
            .map(|i| Uuid::from_u128(i).to_string())
            .collect::<Vec<_>>()
            .join(",");
        for raw in ["nope", ",", hundred_and_one.as_str()] {
            let request = parse_list_query::<PortalLayoutListParams, PortalLayoutSortField>(
                &format!("ids={raw}"),
            )
            .expect("ids is a string");
            let error = PortalLayoutFilter::try_from(request.filter).expect_err(raw);
            assert!(format!("{error:?}").contains("ids"), "{error:?}");
        }
    }
}

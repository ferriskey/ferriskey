use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    list_query::{ListQuery, PaginationParams, parse_id_list},
    paginated::Paginated,
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    common::pagination::{DateRange, PageRequest},
    organization::ports::{
        Organization, OrganizationFilter, OrganizationService, OrganizationSortField,
    },
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct OrganizationListParams {
    pub search: Option<String>,
    pub name: Option<String>,
    pub alias: Option<String>,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub has_domain: Option<bool>,
    pub without_member: Option<Uuid>,
    #[param(example = "0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e6f,0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e70")]
    pub ids: Option<String>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
}

impl TryFrom<OrganizationListParams> for OrganizationFilter {
    type Error = ApiError;

    fn try_from(params: OrganizationListParams) -> Result<Self, Self::Error> {
        Ok(Self {
            search: params.search,
            name: params.name,
            alias: params.alias,
            domain: params.domain,
            description: params.description,
            enabled: params.enabled,
            has_domain: params.has_domain,
            without_member: params.without_member,
            ids: params
                .ids
                .as_deref()
                .map(|raw| parse_id_list("ids", raw))
                .transpose()?,
            created: DateRange::new(params.created_from, params.created_to),
        })
    }
}

#[utoipa::path(
    get,
    path = "",
    tag = "organization",
    summary = "List organizations in a realm",
    description = "Returns one page of the realm's organizations. Text filters (name, alias, domain, description) match case-insensitively anywhere in the value, and an organization without a description never matches a description filter; search matches case-insensitively an organization whose name, alias, domain or description contains the value, and a missing domain or description never matches; enabled matches exactly; has_domain keeps organizations with a domain (true) or without one (false); without_member takes a user id and keeps the organizations that user has not joined; ids takes a comma-separated list of at most 100 organization ids; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        PaginationParams,
        OrganizationListParams,
        ("order_by" = inline(Option<OrganizationSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of organizations", body = Paginated<Organization>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_organizations(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<OrganizationListParams, OrganizationSortField>,
) -> Result<Response<Paginated<Organization>>, ApiError> {
    let request = PageRequest {
        filter: OrganizationFilter::try_from(request.filter)?,
        page: request.page,
        limit: request.limit,
        sort: request.sort,
    };
    let page = state
        .service
        .list_organizations(identity, realm_name, request)
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
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let member = Uuid::new_v4();
        let request = parse_list_query::<OrganizationListParams, OrganizationSortField>(&format!(
            "order_by=alias&order=asc&search=ac&name=acme&alias=ac-1&domain=corp&description=sales&enabled=false&has_domain=true&without_member={member}&ids={first},{second}&created_from=2026-01-01T00:00:00Z&created_to=2026-02-01T00:00:00%2B02:00"
        ))
        .expect("valid query");

        assert_eq!(request.sort.field, OrganizationSortField::Alias);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            OrganizationFilter::try_from(request.filter).expect("valid filter"),
            OrganizationFilter {
                search: Some("ac".to_string()),
                name: Some("acme".to_string()),
                alias: Some("ac-1".to_string()),
                domain: Some("corp".to_string()),
                description: Some("sales".to_string()),
                enabled: Some(false),
                has_domain: Some(true),
                without_member: Some(member),
                ids: Some(vec![first, second]),
                created: DateRange::new(
                    Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                    Utc.with_ymd_and_hms(2026, 1, 31, 22, 0, 0).single(),
                ),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["name", "alias", "enabled", "created_at", "updated_at"] {
            assert!(
                parse_list_query::<OrganizationListParams, OrganizationSortField>(&format!(
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
            "redirect_url=x",
            "order_by=domain",
            "order_by=description",
            "search=a&search=b",
            "description=a&description=b",
            "created_from=2026-10-05",
            "created_to=2026-10-05",
            "enabled=maybe",
            "has_domain=maybe",
            "without_member=nope",
        ] {
            assert!(
                parse_list_query::<OrganizationListParams, OrganizationSortField>(query).is_err(),
                "{query}"
            );
        }
    }

    #[test]
    fn invalid_ids_name_the_parameter() {
        for ids in ["nope".to_string(), ",".to_string()] {
            let request = parse_list_query::<OrganizationListParams, OrganizationSortField>(
                &format!("ids={ids}"),
            )
            .expect("ids is read as text");
            assert!(
                matches!(
                    OrganizationFilter::try_from(request.filter),
                    Err(ApiError::BadRequest(ref body)) if body.message.contains("ids")
                ),
                "{ids}"
            );
        }
    }
}

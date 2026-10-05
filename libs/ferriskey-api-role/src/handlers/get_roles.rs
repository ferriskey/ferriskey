use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    list_query::{ListQuery, PaginationParams, parse_id_list},
    paginated::Paginated,
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::common::pagination::PageRequest;
use ferriskey_core::domain::role::entities::{Role, RoleFilter, RoleScope, RoleSortField};
use ferriskey_core::domain::role::ports::RoleService;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct RoleListParams {
    #[param(example = "app-web.admin")]
    pub search: Option<String>,
    pub name: Option<String>,
    #[param(example = "app-web.admin")]
    pub qualified_name: Option<String>,
    pub description: Option<String>,
    pub require_mfa: Option<bool>,
    pub client_id: Option<Uuid>,
    #[param(inline)]
    pub scope: Option<RoleScope>,
    pub has_permissions: Option<bool>,
    #[param(example = "0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e6f,0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e70")]
    pub ids: Option<String>,
}

impl TryFrom<RoleListParams> for RoleFilter {
    type Error = ApiError;

    fn try_from(params: RoleListParams) -> Result<Self, Self::Error> {
        Ok(Self {
            search: params.search,
            name: params.name,
            qualified_name: params.qualified_name,
            description: params.description,
            require_mfa: params.require_mfa,
            client_id: params.client_id,
            scope: params.scope,
            has_permissions: params.has_permissions,
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
    summary = "List the roles of a realm",
    description = "Returns one page of the realm's roles. search matches case-insensitively a role whose name contains the value, a client role whose client identifier (client_id string) contains the value, or a client role whose qualified value `<client identifier>.<role name>` contains the value (so `app-web.admin` finds the `admin` role of client `app-web`, and `app-web.ns.admin` finds its `ns.admin` role). Text filters (name, description) match case-insensitively anywhere in the value; qualified_name matches exactly and case-sensitively a realm role whose name equals the value or a client role whose `<client identifier>.<role name>` equals the value, so a realm role `a.b` and the `b` role of client `a` are both returned for `a.b`; require_mfa and client_id match exactly; scope keeps realm roles (no client) or client roles; has_permissions keeps roles granting at least one permission (true) or none (false); ids takes a comma-separated list of at most 100 role ids. Filters combine with AND.",
    path = "",
    tag = "role",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        PaginationParams,
        RoleListParams,
        ("order_by" = inline(Option<RoleSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of roles", body = Paginated<Role>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_roles(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<RoleListParams, RoleSortField>,
) -> Result<Response<Paginated<Role>>, ApiError> {
    let request = PageRequest {
        filter: RoleFilter::try_from(request.filter)?,
        page: request.page,
        limit: request.limit,
        sort: request.sort,
    };
    let page = state
        .service
        .list_roles(identity, realm_name, request)
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
        let client_id = Uuid::new_v4();
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let request = parse_list_query::<RoleListParams, RoleSortField>(&format!(
            "order_by=name&order=asc&search=app-web.adm&name=adm&qualified_name=app-web.ns.admin&description=ops&require_mfa=true&client_id={client_id}&scope=client&has_permissions=false&ids={first},{second}"
        ))
        .expect("valid query");

        assert_eq!(request.sort.field, RoleSortField::Name);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            RoleFilter::try_from(request.filter).expect("valid filter"),
            RoleFilter {
                search: Some("app-web.adm".to_string()),
                name: Some("adm".to_string()),
                qualified_name: Some("app-web.ns.admin".to_string()),
                description: Some("ops".to_string()),
                require_mfa: Some(true),
                client_id: Some(client_id),
                scope: Some(RoleScope::Client),
                has_permissions: Some(false),
                ids: Some(vec![first, second]),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["name", "created_at", "updated_at"] {
            assert!(
                parse_list_query::<RoleListParams, RoleSortField>(&format!("order_by={value}"))
                    .is_ok(),
                "{value}"
            );
        }
    }

    #[test]
    fn unknown_filters_and_columns_are_refused() {
        for query in [
            "permissions=x",
            "order_by=permissions",
            "client_id=nope",
            "scope=global",
            "search=a&search=b",
        ] {
            assert!(
                parse_list_query::<RoleListParams, RoleSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        list_query::{ListQuery, PaginationParams, parse_id_list},
        paginated::Paginated,
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::common::pagination::{DateRange, PageRequest};
use ferriskey_core::domain::user::entities::{User, UserFilter, UserSortField};
use ferriskey_core::domain::user::ports::UserService;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct UserListParams {
    pub search: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub enabled: Option<bool>,
    pub email_verified: Option<bool>,
    pub service_account: Option<bool>,
    pub role_id: Option<Uuid>,
    #[param(example = "0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e6f,0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e70")]
    pub ids: Option<String>,
    pub not_in_group: Option<Uuid>,
    pub not_in_organization: Option<Uuid>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
}

impl TryFrom<UserListParams> for UserFilter {
    type Error = ApiError;

    fn try_from(params: UserListParams) -> Result<Self, Self::Error> {
        Ok(Self {
            search: params.search,
            username: params.username,
            email: params.email,
            firstname: params.firstname,
            lastname: params.lastname,
            enabled: params.enabled,
            email_verified: params.email_verified,
            service_account: params.service_account,
            role_id: params.role_id,
            ids: params
                .ids
                .as_deref()
                .map(|raw| parse_id_list("ids", raw))
                .transpose()?,
            not_in_group: params.not_in_group,
            not_in_organization: params.not_in_organization,
            created: DateRange::new(params.created_from, params.created_to),
        })
    }
}

#[utoipa::path(
    get,
    path = "",
    tag = "user",
    summary = "List the users of a realm",
    description = "Returns one page of the realm's users. search matches case-insensitively anywhere in the username, email, firstname or lastname. created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Text filters (username, email, firstname, lastname) match case-insensitively anywhere in the value; enabled, email_verified, service_account and role_id match exactly; ids takes a comma-separated list of at most 100 user ids; not_in_group takes an organization group id and keeps the users that are not members of that group; not_in_organization takes an organization id and keeps the users that are not members of that organization. Filters combine with AND.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        PaginationParams,
        UserListParams,
        ("order_by" = inline(Option<UserSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    responses(
        (status = 200, description = "One page of users", body = Paginated<User>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_users(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<UserListParams, UserSortField>,
) -> Result<Response<Paginated<User>>, ApiError> {
    let request = PageRequest {
        filter: UserFilter::try_from(request.filter)?,
        page: request.page,
        limit: request.limit,
        sort: request.sort,
    };
    let page = state
        .service
        .list_users(identity, realm_name, request)
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
        let role_id = Uuid::new_v4();
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let group_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let request = parse_list_query::<UserListParams, UserSortField>(&format!(
            "order_by=updated_at&order=asc&search=al&created_from=2026-01-01T00:00:00Z&created_to=2026-02-01T00:00:00%2B02:00&username=jo&email=ex&firstname=a&lastname=b&enabled=true&email_verified=false&service_account=true&role_id={role_id}&ids={first},{second}&not_in_group={group_id}&not_in_organization={organization_id}"
        ))
        .expect("valid query");

        assert_eq!(request.sort.field, UserSortField::UpdatedAt);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            UserFilter::try_from(request.filter).expect("valid filter"),
            UserFilter {
                search: Some("al".to_string()),
                username: Some("jo".to_string()),
                email: Some("ex".to_string()),
                firstname: Some("a".to_string()),
                lastname: Some("b".to_string()),
                enabled: Some(true),
                email_verified: Some(false),
                service_account: Some(true),
                role_id: Some(role_id),
                ids: Some(vec![first, second]),
                not_in_group: Some(group_id),
                not_in_organization: Some(organization_id),
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
            "username",
            "email",
            "firstname",
            "lastname",
            "enabled",
            "created_at",
            "updated_at",
        ] {
            assert!(
                parse_list_query::<UserListParams, UserSortField>(&format!("order_by={value}"))
                    .is_ok(),
                "{value}"
            );
        }
    }

    #[test]
    fn unknown_filters_and_columns_are_refused() {
        for query in [
            "password=x",
            "order_by=password",
            "role_id=nope",
            "not_in_group=nope",
            "not_in_organization=nope",
            "created_from=2026-10-05",
            "created_to=2026-10-05",
        ] {
            assert!(
                parse_list_query::<UserListParams, UserSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

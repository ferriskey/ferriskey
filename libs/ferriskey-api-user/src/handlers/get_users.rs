use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        list_query::{ListQuery, PaginationParams},
        paginated::Paginated,
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::{User, UserFilter, UserSortField};
use ferriskey_core::domain::user::ports::UserService;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct UserListParams {
    pub username: Option<String>,
    pub email: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub enabled: Option<bool>,
    pub email_verified: Option<bool>,
    pub service_account: Option<bool>,
    pub role_id: Option<Uuid>,
}

impl From<UserListParams> for UserFilter {
    fn from(params: UserListParams) -> Self {
        Self {
            username: params.username,
            email: params.email,
            firstname: params.firstname,
            lastname: params.lastname,
            enabled: params.enabled,
            email_verified: params.email_verified,
            service_account: params.service_account,
            role_id: params.role_id,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    tag = "user",
    summary = "List the users of a realm",
    description = "Returns one page of the realm's users. Text filters (username, email, firstname, lastname) match case-insensitively anywhere in the value; enabled, email_verified, service_account and role_id match exactly. Filters combine with AND.",
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
    let page = state
        .service
        .list_users(identity, realm_name, request.map_filter(UserFilter::from))
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
        let role_id = Uuid::new_v4();
        let request = parse_list_query::<UserListParams, UserSortField>(&format!(
            "order_by=updated_at&order=asc&username=jo&email=ex&firstname=a&lastname=b&enabled=true&email_verified=false&service_account=true&role_id={role_id}"
        ))
        .expect("valid query")
        .map_filter(UserFilter::from);

        assert_eq!(request.sort.field, UserSortField::UpdatedAt);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            request.filter,
            UserFilter {
                username: Some("jo".to_string()),
                email: Some("ex".to_string()),
                firstname: Some("a".to_string()),
                lastname: Some("b".to_string()),
                enabled: Some(true),
                email_verified: Some(false),
                service_account: Some(true),
                role_id: Some(role_id),
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
        for query in ["password=x", "order_by=password", "role_id=nope"] {
            assert!(
                parse_list_query::<UserListParams, UserSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

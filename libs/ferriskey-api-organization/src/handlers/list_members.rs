use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    common::pagination::PageRequest,
    organization::ports::{
        ListOrganizationMembersInput, OrganizationId, OrganizationMember, OrganizationMemberFilter,
        OrganizationMemberSortField, OrganizationService,
    },
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    list_query::{ListQuery, PaginationParams},
    paginated::Paginated,
    response::Response,
};
use ferriskey_api_core::app_state::AppState;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct OrganizationMemberListParams {
    pub username: Option<String>,
    pub email: Option<String>,
    pub enabled: Option<bool>,
}

impl From<OrganizationMemberListParams> for OrganizationMemberFilter {
    fn from(params: OrganizationMemberListParams) -> Self {
        Self {
            username: params.username,
            email: params.email,
            enabled: params.enabled,
        }
    }
}

#[utoipa::path(
    get,
    path = "/{organization_id}/members",
    tag = "organization",
    summary = "List members of an organization",
    description = "Returns one page of the organization's memberships. Text filters (username, email) match case-insensitively anywhere in the member's user value; enabled matches the user's enabled flag exactly. Filters combine with AND. created_at is the date the user joined the organization.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("organization_id" = Uuid, Path, description = "Organization ID"),
        PaginationParams,
        OrganizationMemberListParams,
        ("order_by" = inline(Option<OrganizationMemberSortField>), Query, description = "Sort column, `created_at` (membership date) by default"),
    ),
    responses(
        (status = 200, description = "One page of organization members", body = Paginated<OrganizationMember>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Organization not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn list_members(
    Path((realm_name, organization_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<OrganizationMemberListParams, OrganizationMemberSortField>,
) -> Result<Response<Paginated<OrganizationMember>>, ApiError> {
    let request = PageRequest {
        filter: OrganizationMemberFilter::from(request.filter),
        page: request.page,
        limit: request.limit,
        sort: request.sort,
    };
    let page = state
        .service
        .list_members(
            identity,
            ListOrganizationMembersInput {
                realm_name,
                organization_id: OrganizationId::new(organization_id),
            },
            request,
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
        let request =
            parse_list_query::<OrganizationMemberListParams, OrganizationMemberSortField>(
                "order_by=username&order=asc&username=jo&email=corp&enabled=false",
            )
            .expect("valid query");

        assert_eq!(request.sort.field, OrganizationMemberSortField::Username);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            OrganizationMemberFilter::from(request.filter),
            OrganizationMemberFilter {
                username: Some("jo".to_string()),
                email: Some("corp".to_string()),
                enabled: Some(false),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["username", "email", "created_at"] {
            assert!(
                parse_list_query::<OrganizationMemberListParams, OrganizationMemberSortField>(
                    &format!("order_by={value}")
                )
                .is_ok(),
                "{value}"
            );
        }
    }

    #[test]
    fn unknown_filters_and_columns_are_refused() {
        for query in [
            "search=jo",
            "user_id=x",
            "order_by=firstname",
            "enabled=maybe",
        ] {
            assert!(
                parse_list_query::<OrganizationMemberListParams, OrganizationMemberSortField>(
                    query
                )
                .is_err(),
                "{query}"
            );
        }
    }
}

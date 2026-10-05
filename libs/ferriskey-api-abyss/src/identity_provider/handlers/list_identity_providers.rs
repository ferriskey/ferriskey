use crate::identity_provider::dto::{IdentityProviderListParams, IdentityProviderResponse};
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
use ferriskey_core::domain::abyss::identity_provider::{
    IdentityProviderSortField, entities::ListIdentityProvidersInput, ports::IdentityProviderService,
};
use ferriskey_core::domain::authentication::value_objects::Identity;

#[utoipa::path(
    get,
    path = "/identity-providers",
    summary = "List the identity providers of a realm",
    description = "Returns one page of the realm's identity providers. search matches case-insensitively a provider whose alias or display_name contains the value; a provider without a display name only matches by alias. Text filters (alias, display_name) match case-insensitively anywhere in the value; provider_id and enabled match exactly; health is the configuration status shown by the console: error when client_id, client_secret, authorization_url or token_url is missing, degraded when scopes is missing, healthy otherwise; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. Client secrets stay masked.",
    responses(
        (status = 200, body = Paginated<IdentityProviderResponse>, description = "One page of identity providers"),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "The name of the realm"),
        PaginationParams,
        IdentityProviderListParams,
        ("order_by" = inline(Option<IdentityProviderSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    tag = "identity_provider",
)]
pub async fn list_identity_providers(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<IdentityProviderListParams, IdentityProviderSortField>,
) -> Result<Response<Paginated<IdentityProviderResponse>>, ApiError> {
    let page = state
        .service
        .list_identity_providers(
            identity,
            ListIdentityProvidersInput { realm_name },
            request.map_filter(Into::into),
        )
        .await?;

    Ok(Response::OK(Paginated::from(
        page.map(IdentityProviderResponse::from),
    )))
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use ferriskey_api_core::api_entities::list_query::parse_list_query;
    use ferriskey_core::domain::abyss::identity_provider::{
        IdentityProviderFilter, IdentityProviderHealth,
    };
    use ferriskey_core::domain::common::pagination::{DateRange, SortOrder};

    use super::*;

    #[test]
    fn every_filter_and_sort_field_is_read() {
        let request = parse_list_query::<IdentityProviderListParams, IdentityProviderSortField>(
            "order_by=display_name&order=asc&search=gi&alias=git&display_name=Hub&provider_id=oidc&enabled=false&health=degraded&created_from=2026-01-01T00:00:00Z&created_to=2026-02-01T00:00:00%2B02:00",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, IdentityProviderSortField::DisplayName);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            IdentityProviderFilter::from(request.filter),
            IdentityProviderFilter {
                search: Some("gi".to_string()),
                alias: Some("git".to_string()),
                display_name: Some("Hub".to_string()),
                provider_id: Some("oidc".to_string()),
                enabled: Some(false),
                health: Some(IdentityProviderHealth::Degraded),
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
            "alias",
            "display_name",
            "provider_id",
            "enabled",
            "created_at",
            "updated_at",
        ] {
            assert!(
                parse_list_query::<IdentityProviderListParams, IdentityProviderSortField>(
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
            "brief_representation=true",
            "order_by=config",
            "health=broken",
            "enabled=maybe",
            "search=a&search=b",
            "created_from=2026-10-05",
            "created_to=2026-10-05",
        ] {
            assert!(
                parse_list_query::<IdentityProviderListParams, IdentityProviderSortField>(query)
                    .is_err(),
                "{query}"
            );
        }
    }
}

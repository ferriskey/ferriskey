use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    abyss::federation::{entities::FederationProviderSortField, ports::FederationService},
    authentication::value_objects::Identity,
};

use crate::federation::dto::{FederationProviderListParams, ProviderResponse};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    list_query::{ListQuery, PaginationParams},
    paginated::Paginated,
    response::Response,
};
use ferriskey_api_core::app_state::AppState;

#[utoipa::path(
    get,
    path = "/federation/providers",
    summary = "List the federation providers of a realm",
    description = "Returns one page of the realm's user federation providers. The name filter matches case-insensitively anywhere in the value; provider_type, enabled and sync_enabled match exactly; synced=true keeps providers that have synchronized at least once, synced=false keeps the ones that never did. Filters combine with AND. Sorting on last_sync_at puts never-synchronized providers last in ascending order and first in descending order. Bind credentials stay masked.",
    responses(
        (status = 200, description = "One page of federation providers", body = Paginated<ProviderResponse>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        PaginationParams,
        FederationProviderListParams,
        ("order_by" = inline(Option<FederationProviderSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    tag = "federation"
)]
pub async fn list_providers(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<FederationProviderListParams, FederationProviderSortField>,
) -> Result<Response<Paginated<ProviderResponse>>, ApiError> {
    let page = state
        .service
        .list_federation_providers(identity, realm_name, request.map_filter(Into::into))
        .await?;

    Ok(Response::OK(Paginated::from(
        page.map(ProviderResponse::from),
    )))
}

#[cfg(test)]
mod tests {
    use ferriskey_api_core::api_entities::list_query::parse_list_query;
    use ferriskey_core::domain::abyss::federation::entities::FederationProviderFilter;
    use ferriskey_core::domain::common::pagination::SortOrder;

    use super::*;

    #[test]
    fn every_filter_and_sort_field_is_read() {
        let request = parse_list_query::<FederationProviderListParams, FederationProviderSortField>(
            "order_by=last_sync_at&order=asc&name=corp&provider_type=Ldap&enabled=false&sync_enabled=true&synced=false",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, FederationProviderSortField::LastSyncAt);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            FederationProviderFilter::from(request.filter),
            FederationProviderFilter {
                name: Some("corp".to_string()),
                provider_type: Some("Ldap".to_string()),
                enabled: Some(false),
                sync_enabled: Some(true),
                synced: Some(false),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in [
            "name",
            "priority",
            "enabled",
            "last_sync_at",
            "created_at",
            "updated_at",
        ] {
            assert!(
                parse_list_query::<FederationProviderListParams, FederationProviderSortField>(
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
            "search=corp",
            "order_by=config",
            "order_by=sync_mode",
            "enabled=maybe",
            "synced=never",
        ] {
            assert!(
                parse_list_query::<FederationProviderListParams, FederationProviderSortField>(
                    query
                )
                .is_err(),
                "{query}"
            );
        }
    }
}

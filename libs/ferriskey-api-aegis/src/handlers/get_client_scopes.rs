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
use ferriskey_core::domain::aegis::entities::{
    ClientScope, ClientScopeFilter, ClientScopeSortField, ScopeType,
};
use ferriskey_core::domain::aegis::ports::ClientScopeService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct ClientScopeListParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub search: Option<String>,
    #[param(example = "openid-connect")]
    pub protocol: Option<String>,
    #[param(inline)]
    pub default_scope_type: Option<ScopeType>,
    pub has_protocol_mappers: Option<bool>,
    #[param(example = "0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e6f")]
    pub not_assigned_to_client: Option<Uuid>,
}

impl From<ClientScopeListParams> for ClientScopeFilter {
    fn from(params: ClientScopeListParams) -> Self {
        Self {
            name: params.name,
            description: params.description,
            search: params.search,
            protocol: params.protocol,
            default_scope_type: params.default_scope_type,
            has_protocol_mappers: params.has_protocol_mappers,
            not_assigned_to_client: params.not_assigned_to_client,
        }
    }
}

#[utoipa::path(
    get,
    path = "/client-scopes",
    summary = "List the client scopes of a realm",
    description = "Returns one page of the realm's client scopes, each with its protocol mappers. Text filters (name, description) match case-insensitively anywhere in the value; search matches the name or the description; protocol and default_scope_type match exactly; has_protocol_mappers keeps scopes with at least one protocol mapper (true) or none (false); not_assigned_to_client takes a client id and keeps the scopes not yet assigned to that client. Filters combine with AND.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        PaginationParams,
        ClientScopeListParams,
        ("order_by" = inline(Option<ClientScopeSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    tag = "client-scope",
    responses(
        (status = 200, description = "One page of client scopes", body = Paginated<ClientScope>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_client_scopes(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<ClientScopeListParams, ClientScopeSortField>,
) -> Result<Response<Paginated<ClientScope>>, ApiError> {
    let page = state
        .service
        .list_client_scopes(
            identity,
            realm_name,
            request.map_filter(ClientScopeFilter::from),
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
        let request = parse_list_query::<ClientScopeListParams, ClientScopeSortField>(
            "order_by=name&order=asc&name=pro&description=claims&search=mail&protocol=saml&default_scope_type=OPTIONAL&has_protocol_mappers=false&not_assigned_to_client=0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e6f",
        )
        .expect("valid query");

        assert_eq!(request.sort.field, ClientScopeSortField::Name);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            ClientScopeFilter::from(request.filter),
            ClientScopeFilter {
                name: Some("pro".to_string()),
                description: Some("claims".to_string()),
                search: Some("mail".to_string()),
                protocol: Some("saml".to_string()),
                default_scope_type: Some(ScopeType::Optional),
                has_protocol_mappers: Some(false),
                not_assigned_to_client: Some(
                    Uuid::parse_str("0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e6f").expect("uuid"),
                ),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["name", "created_at", "updated_at"] {
            assert!(
                parse_list_query::<ClientScopeListParams, ClientScopeSortField>(&format!(
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
            "order_by=protocol",
            "default_scope_type=optional",
            "has_protocol_mappers=maybe",
            "not_assigned_to_client=nope",
        ] {
            assert!(
                parse_list_query::<ClientScopeListParams, ClientScopeSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

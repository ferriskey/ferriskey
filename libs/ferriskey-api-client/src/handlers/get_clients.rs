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
use ferriskey_core::domain::authentication::entities::AuthProtocol;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::{
    ApplicationType, Client, ClientFilter, ClientSortField, ClientType,
};
use ferriskey_core::domain::client::ports::ClientService;
use ferriskey_core::domain::common::pagination::PageRequest;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct ClientListParams {
    pub search: Option<String>,
    pub name: Option<String>,
    pub client_id: Option<String>,
    pub enabled: Option<bool>,
    pub public_client: Option<bool>,
    pub service_account_enabled: Option<bool>,
    pub oauth_device_code_grant_enabled: Option<bool>,
    #[param(inline)]
    pub protocol: Option<AuthProtocol>,
    #[param(inline)]
    pub client_type: Option<ClientType>,
    #[param(inline)]
    pub application_type: Option<ApplicationType>,
    pub has_redirect_uris: Option<bool>,
    pub maintenance_enabled: Option<bool>,
    #[param(example = "0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e6f,0199a4f2-3c1e-7b8a-9f00-1a2b3c4d5e70")]
    pub ids: Option<String>,
}

impl TryFrom<ClientListParams> for ClientFilter {
    type Error = ApiError;

    fn try_from(params: ClientListParams) -> Result<Self, Self::Error> {
        Ok(Self {
            search: params.search,
            name: params.name,
            client_id: params.client_id,
            enabled: params.enabled,
            public_client: params.public_client,
            service_account_enabled: params.service_account_enabled,
            oauth_device_code_grant_enabled: params.oauth_device_code_grant_enabled,
            protocol: params.protocol,
            client_type: params.client_type,
            application_type: params.application_type,
            has_redirect_uris: params.has_redirect_uris,
            maintenance_enabled: params.maintenance_enabled,
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
    summary = "List the clients of a realm",
    description = "Returns one page of the realm's clients. search matches case-insensitively anywhere in the name or the client_id; the text filters name and client_id match case-insensitively anywhere in that value; enabled, public_client, service_account_enabled, protocol and client_type match exactly; oauth_device_code_grant_enabled treats an unset value as false; application_type keeps m2m clients (service account), device clients (device grant and no redirect URI), spa and native clients (public client type, with or without public_client) and web clients (any other client type); has_redirect_uris keeps clients with at least one redirect URI (true) or none (false), disabled redirect URIs included; maintenance_enabled keeps clients in maintenance (true) or not (false); ids takes a comma-separated list of at most 100 client ids. Filters combine with AND.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        PaginationParams,
        ClientListParams,
        ("order_by" = inline(Option<ClientSortField>), Query, description = "Sort column, `created_at` by default"),
    ),
    tag = "client",
    responses(
        (status = 200, description = "One page of clients", body = Paginated<Client>),
        (status = 400, description = "Invalid query parameter", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_clients(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ListQuery(request): ListQuery<ClientListParams, ClientSortField>,
) -> Result<Response<Paginated<Client>>, ApiError> {
    let request = PageRequest {
        filter: ClientFilter::try_from(request.filter)?,
        page: request.page,
        limit: request.limit,
        sort: request.sort,
    };
    let page = state
        .service
        .list_clients(identity, realm_name, request)
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
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let request = parse_list_query::<ClientListParams, ClientSortField>(&format!(
            "order_by=client_id&order=asc&search=po&name=web&client_id=app&enabled=true&public_client=false&service_account_enabled=true&oauth_device_code_grant_enabled=false&protocol=openid-connect&client_type=public&application_type=native&has_redirect_uris=false&maintenance_enabled=true&ids={first},{second}"
        ))
        .expect("valid query");

        assert_eq!(request.sort.field, ClientSortField::ClientId);
        assert_eq!(request.sort.order, SortOrder::Asc);
        assert_eq!(
            ClientFilter::try_from(request.filter).expect("valid filter"),
            ClientFilter {
                search: Some("po".to_string()),
                name: Some("web".to_string()),
                client_id: Some("app".to_string()),
                enabled: Some(true),
                public_client: Some(false),
                service_account_enabled: Some(true),
                oauth_device_code_grant_enabled: Some(false),
                protocol: Some(AuthProtocol::OpenIdConnect),
                client_type: Some(ClientType::Public),
                application_type: Some(ApplicationType::Native),
                has_redirect_uris: Some(false),
                maintenance_enabled: Some(true),
                ids: Some(vec![first, second]),
            }
        );
    }

    #[test]
    fn every_documented_sort_value_parses() {
        for value in ["name", "client_id", "enabled", "created_at", "updated_at"] {
            assert!(
                parse_list_query::<ClientListParams, ClientSortField>(&format!("order_by={value}"))
                    .is_ok(),
                "{value}"
            );
        }
    }

    #[test]
    fn unknown_filters_and_columns_are_refused() {
        for query in [
            "secret=x",
            "order_by=secret",
            "protocol=oauth",
            "client_type=robot",
            "application_type=robot",
            "enabled=maybe",
        ] {
            assert!(
                parse_list_query::<ClientListParams, ClientSortField>(query).is_err(),
                "{query}"
            );
        }
    }
}

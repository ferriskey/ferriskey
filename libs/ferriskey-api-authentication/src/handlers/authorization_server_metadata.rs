use axum::extract::{Path, State};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::url::{FullUrl, root_scoped_base_url};
use ferriskey_core::domain::realm::ports::RealmService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn realm_issuer(base_url: &str, root_path: &str, realm_name: &str) -> String {
    format!(
        "{}/realms/{realm_name}",
        root_scoped_base_url(base_url, root_path)
    )
}

pub fn supported_grant_types() -> Vec<String> {
    [
        "authorization_code",
        "refresh_token",
        "client_credentials",
        "password",
        "urn:ietf:params:oauth:grant-type:device_code",
        "urn:ietf:params:oauth:grant-type:token-exchange",
    ]
    .map(String::from)
    .to_vec()
}

pub fn supported_token_endpoint_auth_methods() -> Vec<String> {
    ["none", "client_secret_basic", "client_secret_post"]
        .map(String::from)
        .to_vec()
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct AuthorizationServerMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub revocation_endpoint: String,
    pub introspection_endpoint: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub authorization_response_iss_parameter_supported: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id_metadata_document_supported: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/.well-known/oauth-authorization-server/realms/{realm_name}",
    tag = "auth",
    summary = "Get OAuth 2.0 authorization server metadata",
    description = "RFC 8414 authorization server metadata of a realm, in the path-insertion form. `registration_endpoint` is present only when dynamic client registration is enabled on the realm, `client_id_metadata_document_supported` only when client ID metadata documents are enabled. `scopes_supported` lists the realm's client scopes.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = AuthorizationServerMetadata),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
    )
)]
pub async fn get_authorization_server_metadata(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
) -> Result<Response<AuthorizationServerMetadata>, ApiError> {
    let settings = state
        .service
        .get_authorization_server_settings(realm_name.clone())
        .await?;

    let issuer = realm_issuer(&base_url, &state.args.server.root_path, &realm_name);

    Ok(Response::OK(AuthorizationServerMetadata {
        authorization_endpoint: format!("{issuer}/protocol/openid-connect/auth"),
        token_endpoint: format!("{issuer}/protocol/openid-connect/token"),
        jwks_uri: format!("{issuer}/protocol/openid-connect/jwks.json"),
        revocation_endpoint: format!("{issuer}/protocol/openid-connect/revoke"),
        introspection_endpoint: format!("{issuer}/protocol/openid-connect/token/introspect"),
        scopes_supported: settings.scopes_supported,
        response_types_supported: vec!["code".to_string()],
        grant_types_supported: supported_grant_types(),
        code_challenge_methods_supported: vec!["S256".to_string()],
        token_endpoint_auth_methods_supported: supported_token_endpoint_auth_methods(),
        authorization_response_iss_parameter_supported: true,
        registration_endpoint: settings
            .dcr_enabled
            .then(|| format!("{issuer}/clients/register")),
        client_id_metadata_document_supported: settings.cimd_enabled.then_some(true),
        issuer,
    }))
}

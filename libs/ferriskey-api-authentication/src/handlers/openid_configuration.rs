use super::authorization_server_metadata::{
    realm_issuer, supported_grant_types, supported_token_endpoint_auth_methods,
};
use axum::extract::{Path, State};
use ferriskey_api_core::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
    url::FullUrl,
};
use ferriskey_core::domain::realm::ports::RealmService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct GetOpenIdConfigurationResponse {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub revocation_endpoint: String,
    pub end_session_endpoint: String,
    pub introspection_endpoint: String,
    pub device_authorization_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub grant_types_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    /// OIDC Back-Channel Logout 1.0: clients may register a
    /// `backchannel_logout_uri` to be told when a session ends.
    pub backchannel_logout_supported: bool,
    /// Logout tokens carry the `sid` of the session that ended.
    pub backchannel_logout_session_supported: bool,
    pub scopes_supported: Vec<String>,
    pub authorization_response_iss_parameter_supported: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id_metadata_document_supported: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/.well-known/openid-configuration",
    tag = "auth",
    summary = "Get OpenID Connect configuration",
    description = "Retrieves the OpenID Connect configuration for a specific realm. This endpoint provides metadata about the OpenID Connect provider, including endpoints for authorization, token issuance, introspection, user information, and JWKs.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = GetOpenIdConfigurationResponse)
    )
)]
pub async fn get_openid_configuration(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
) -> Result<Response<GetOpenIdConfigurationResponse>, ApiError> {
    let settings = state
        .service
        .get_authorization_server_settings(realm_name.clone())
        .await?;

    let issuer = realm_issuer(&base_url, &state.args.server.root_path, &realm_name);

    Ok(Response::OK(GetOpenIdConfigurationResponse {
        authorization_endpoint: format!("{issuer}/protocol/openid-connect/auth"),
        token_endpoint: format!("{issuer}/protocol/openid-connect/token"),
        revocation_endpoint: format!("{issuer}/protocol/openid-connect/revoke"),
        end_session_endpoint: format!("{issuer}/protocol/openid-connect/logout"),
        introspection_endpoint: format!("{issuer}/protocol/openid-connect/token/introspect"),
        device_authorization_endpoint: format!("{issuer}/protocol/openid-connect/auth/device"),
        userinfo_endpoint: format!("{issuer}/protocol/openid-connect/userinfo"),
        jwks_uri: format!("{issuer}/protocol/openid-connect/jwks.json"),
        grant_types_supported: supported_grant_types(),
        response_types_supported: vec![
            "code".to_string(),
            "none".to_string(),
            "id_token".to_string(),
            "token".to_string(),
            "id_token token".to_string(),
            "code id_token".to_string(),
            "code token".to_string(),
            "code id_token token".to_string(),
        ],
        subject_types_supported: vec!["public".to_string()],
        id_token_signing_alg_values_supported: vec!["RS256".to_string()],
        token_endpoint_auth_methods_supported: supported_token_endpoint_auth_methods(),
        code_challenge_methods_supported: vec!["S256".to_string()],
        backchannel_logout_supported: true,
        backchannel_logout_session_supported: true,
        scopes_supported: settings.scopes_supported,
        authorization_response_iss_parameter_supported: true,
        registration_endpoint: settings
            .dcr_enabled
            .then(|| format!("{issuer}/clients/register")),
        client_id_metadata_document_supported: settings.cimd_enabled.then_some(true),
        issuer,
    }))
}

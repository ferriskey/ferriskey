use super::auth::root_scoped_base_url;
use crate::application::http::server::{api_entities::response::Response, app_state::AppState};
use axum::http::Request;
use axum::{
    body::Body,
    extract::{Path, State},
};
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
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub grant_types_supported: Vec<String>,
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
    req: Request<Body>,
) -> Result<Response<GetOpenIdConfigurationResponse>, String> {
    // Here you would typically fetch the issuer from a database or configuration
    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost");
    let scheme = req.uri().scheme_str().unwrap_or_else(|| {
        req.headers()
            .get("x-forwarded-proto")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("http")
    });

    let base_url = format!("{scheme}://{host}");

    let issuer = format!(
        "{}/realms/{}",
        root_scoped_base_url(&base_url, &state.args.server.root_path),
        realm_name
    );

    Ok(Response::OK(GetOpenIdConfigurationResponse {
        issuer: issuer.clone(),
        authorization_endpoint: format!("{issuer}/protocol/openid-connect/auth"),
        token_endpoint: format!("{issuer}/protocol/openid-connect/token"),
        revocation_endpoint: format!("{issuer}/protocol/openid-connect/revoke"),
        end_session_endpoint: format!("{issuer}/protocol/openid-connect/logout"),
        introspection_endpoint: format!("{issuer}/protocol/openid-connect/token/introspect"),
        userinfo_endpoint: format!("{issuer}/protocol/openid-connect/userinfo"),
        jwks_uri: format!("{issuer}/protocol/openid-connect/certs"),
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
            "client_credentials".to_string(),
            "password".to_string(),
        ],
    }))
}

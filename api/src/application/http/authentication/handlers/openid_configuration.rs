use crate::application::http::server::api_entities::response::Response;
use axum::http::Request;
use axum::{body::Body, extract::Path};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct GetOpenIdConfigurationResponse {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
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
    let issuer = format!("{base_url}/realms/{realm_name}");

    Ok(Response::OK(GetOpenIdConfigurationResponse {
        issuer: issuer.clone(),
        authorization_endpoint: format!("{issuer}/protocol/openid-connect/auth"),
        token_endpoint: format!("{issuer}/protocol/openid-connect/token"),
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

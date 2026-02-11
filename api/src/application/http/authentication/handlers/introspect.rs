use axum::{
    Form,
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
};
use base64::{Engine, engine::general_purpose};
use ferriskey_core::domain::authentication::{
    entities::TokenIntrospectionResponse, ports::AuthService, value_objects::IntrospectTokenInput,
};
use validator::Validate;

use crate::application::http::authentication::validators::IntrospectRequestValidator;
use crate::application::http::server::{api_entities::api_error::ApiError, app_state::AppState};

fn try_parse_basic_client_credentials(headers: &HeaderMap) -> Option<(String, String)> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let value = value.strip_prefix("Basic ")?;

    let decoded = general_purpose::STANDARD.decode(value).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;

    let (client_id, client_secret) = decoded.split_once(':')?;
    Some((client_id.to_string(), client_secret.to_string()))
}

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/token/introspect",
    tag = "auth",
    summary = "Token introspection",
    description = "OAuth2/OIDC Token Introspection (RFC 7662). Only confidential clients may call this endpoint using client_secret_basic or client_secret_post. Authorization requires the caller's service account to have the role `introspect` (treated as the `introspect` scope).",
    request_body = IntrospectRequestValidator,
    params(
      ("realm_name" = String, Path, description = "Realm name")
    ),
    responses(
        (status = 200, body = TokenIntrospectionResponse)
    )
)]
pub async fn introspect_token(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(payload): Form<IntrospectRequestValidator>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;

    let (client_id, client_secret) = match try_parse_basic_client_credentials(&headers) {
        Some(creds) => creds,
        None => {
            let client_id = payload.client_id.ok_or_else(|| {
                ApiError::Unauthorized("Missing client authentication".to_string())
            })?;
            let client_secret = payload.client_secret.ok_or_else(|| {
                ApiError::Unauthorized("Missing client authentication".to_string())
            })?;

            (client_id, client_secret)
        }
    };

    let response = state
        .service
        .introspect_token(IntrospectTokenInput {
            realm_name,
            client_id,
            client_secret,
            token: payload.token,
            token_type_hint: payload.token_type_hint,
        })
        .await?;

    Ok(axum::Json(response))
}

#[cfg(test)]
mod tests {
    use super::try_parse_basic_client_credentials;
    use axum::http::{HeaderMap, HeaderValue, header::AUTHORIZATION};

    fn basic(value: &str) -> HeaderValue {
        HeaderValue::from_str(value).unwrap()
    }

    #[test]
    fn parses_basic_credentials() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, basic("Basic Y2xpZW50OnNlY3JldA==")); // client:secret

        let creds = try_parse_basic_client_credentials(&headers);
        assert_eq!(creds, Some(("client".to_string(), "secret".to_string())));
    }

    #[test]
    fn rejects_non_basic_header() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, basic("Bearer token"));
        assert_eq!(try_parse_basic_client_credentials(&headers), None);
    }

    #[test]
    fn rejects_malformed_base64() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, basic("Basic ???"));
        assert_eq!(try_parse_basic_client_credentials(&headers), None);
    }
}

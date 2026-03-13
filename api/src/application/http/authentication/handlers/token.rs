use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::app_state::AppState;
use crate::application::http::{
    authentication::validators::TokenRequestValidator,
    server::api_entities::api_error::ApiErrorResponse,
};
use crate::application::url::FullUrl;
use axum::{
    Form,
    extract::{Path, State},
    http::{
        HeaderValue, StatusCode,
        header::{AUTHORIZATION, SET_COOKIE},
    },
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::{Engine, engine::general_purpose};
use ferriskey_core::domain::authentication::entities::JwtToken;
use ferriskey_core::domain::authentication::{entities::ExchangeTokenInput, ports::AuthService};
use tracing::{instrument, warn};

const IDENTITY_COOKIE: &str = "FERRISKEY_IDENTITY";

fn try_parse_basic_client_credentials(headers: &axum::http::HeaderMap) -> Option<(String, String)> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let prefix = "Basic ";
    if value.len() < prefix.len() || !value[..prefix.len()].eq_ignore_ascii_case(prefix) {
        return None;
    }
    let value = &value[prefix.len()..];

    let decoded = general_purpose::STANDARD.decode(value).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;

    let (client_id, client_secret) = decoded.split_once(':')?;
    Some((client_id.to_string(), client_secret.to_string()))
}

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/token",
    tag = "auth",
    summary = "Exchange token",
    description = "Exchanges authorization grants for JWT tokens. Supports authorization code, client credentials, and refresh token grants. PKCE is required for authorization code grant. HTTP Basic authentication is supported for confidential clients.",
    request_body = TokenRequestValidator,
    params(
      ("realm_name" = String, Path, description = "Realm name")
    ),
    responses(
        (status = 200, body = JwtToken),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
#[instrument(
    skip(state, payload, headers),
    fields(
        realm_name = %realm_name,
        client_id = %payload.client_id,
        grant_type = ?payload.grant_type,
        has_client_secret = payload.client_secret.is_some(),
        has_code = payload.code.is_some(),
        has_refresh_token = payload.refresh_token.is_some(),
        has_code_verifier = payload.code_verifier.is_some()
    )
)]
pub async fn exchange_token(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    headers: axum::http::HeaderMap,
    Form(payload): Form<TokenRequestValidator>,
) -> Result<impl IntoResponse, ApiError> {
    let grant_type = payload.grant_type.clone();
    let payload_client_id = payload.client_id.clone();

    let (client_id, client_secret) = match try_parse_basic_client_credentials(&headers) {
        Some((basic_client_id, basic_client_secret)) => {
            // HTTP Basic authentication takes precedence
            (basic_client_id, Some(basic_client_secret))
        }
        None => {
            // Fall back to client_secret_post (form body)
            (payload_client_id.clone(), payload.client_secret.clone())
        }
    };

    let has_client_secret = client_secret.is_some();
    let has_code = payload.code.is_some();
    let has_refresh_token = payload.refresh_token.is_some();
    let has_code_verifier = payload.code_verifier.is_some();

    let is_secure = base_url.starts_with("https://");
    let token = match state
        .service
        .exchange_token(ExchangeTokenInput {
            realm_name,
            client_id,
            client_secret,
            code: payload.code,
            refresh_token: payload.refresh_token,
            base_url,
            grant_type: payload.grant_type,
            scope: payload.scope,
            code_verifier: payload.code_verifier,
            code_challenge: payload.code_challenge,
            code_challenge_method: payload.code_challenge_method,
        })
        .await
    {
        Ok(token) => token,
        Err(error) => {
            warn!(
                client_id = %payload_client_id,
                grant_type = ?grant_type,
                has_client_secret,
                has_code,
                has_refresh_token,
                has_code_verifier,
                error = ?error,
                "Token exchange failed"
            );
            return Err(error.into());
        }
    };

    let mut identity_cookie = Cookie::build((IDENTITY_COOKIE, token.access_token().to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if is_secure {
        identity_cookie = identity_cookie.secure(true);
    }

    let cookie_value = HeaderValue::from_str(&identity_cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".to_string()))?;

    Ok((
        StatusCode::OK,
        [(SET_COOKIE, cookie_value)],
        axum::Json(token),
    ))
}

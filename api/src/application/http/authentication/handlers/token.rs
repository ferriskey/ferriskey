use super::auth::root_scoped_base_url;
use crate::application::http::authentication::basic_auth::try_parse_basic_client_credentials;
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
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_core::domain::authentication::entities::{GrantType, JwtToken};
use ferriskey_core::domain::authentication::{entities::ExchangeTokenInput, ports::AuthService};
use tracing::{instrument, warn};

const IDENTITY_COOKIE: &str = "FERRISKEY_IDENTITY";

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/token",
    tag = "auth",
    summary = "Exchange token",
    description = "Exchanges a token for a JWT token. This endpoint allows clients to exchange various types of tokens (like authorization codes, refresh tokens, etc.) for a JWT token.",
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
        grant_type = ?payload.grant_type,
        has_username = payload.username.is_some(),
        has_password = payload.password.is_some(),
        has_code = payload.code.is_some(),
        has_refresh_token = payload.refresh_token.is_some()
    )
)]
pub async fn exchange_token(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    headers: HeaderMap,
    Form(payload): Form<TokenRequestValidator>,
) -> Result<impl IntoResponse, ApiError> {
    let (client_id, client_secret) = match try_parse_basic_client_credentials(&headers) {
        Some((id, sec)) => (id, Some(sec)),
        None => (
            payload.client_id.clone().unwrap_or_default(),
            payload.client_secret.clone(),
        ),
    };

    let grant_type = payload.grant_type.clone();
    let has_client_secret = client_secret.is_some();
    let has_username = payload.username.is_some();
    let has_password = payload.password.is_some();
    let has_code = payload.code.is_some();
    let has_refresh_token = payload.refresh_token.is_some();

    let is_secure = base_url.starts_with("https://");
    let base_url = root_scoped_base_url(&base_url, &state.args.server.root_path);

    let exchange_input = ExchangeTokenInput {
        realm_name,
        client_id: client_id.clone(),
        client_secret,
        code: payload.code,
        username: payload.username,
        password: payload.password,
        refresh_token: payload.refresh_token,
        base_url,
        grant_type: payload.grant_type.clone(),
        scope: payload.scope,
        device_code: payload.device_code,
    };

    // The device_code grant is served by the device flow polling path so its
    // RFC 8628 §3.5 error codes survive as an RFC 6749 §5.2 error response.
    let token = if payload.grant_type == GrantType::DeviceCode {
        match state.service.poll_device_token(exchange_input).await {
            Ok(token) => token,
            Err(error) => {
                warn!(client_id = %client_id, error = ?error, "Device token poll failed");
                return Err(error.into());
            }
        }
    } else {
        match state.service.exchange_token(exchange_input).await {
            Ok(token) => token,
            Err(error) => {
                warn!(
                    client_id = %client_id,
                    grant_type = ?grant_type,
                    has_client_secret,
                    has_username,
                    has_password,
                    has_code,
                    has_refresh_token,
                    error = ?error,
                    "Token exchange failed"
                );
                return Err(error.into());
            }
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
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))?;

    Ok((
        StatusCode::OK,
        [(SET_COOKIE, cookie_value)],
        axum::Json(token),
    ))
}

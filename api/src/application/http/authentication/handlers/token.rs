use crate::application::http::authentication::validators::TokenRequestValidator;
use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::app_state::AppState;
use crate::application::url::FullUrl;
use axum::{
    Form,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_core::domain::authentication::entities::JwtToken;
use ferriskey_core::domain::authentication::{entities::ExchangeTokenInput, ports::AuthService};

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
        (status = 200, body = JwtToken)
    )
)]
pub async fn exchange_token(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    Form(payload): Form<TokenRequestValidator>,
) -> Result<impl IntoResponse, ApiError> {
    let is_secure = base_url.starts_with("https://");
    let token = state
        .service
        .exchange_token(ExchangeTokenInput {
            realm_name,
            client_id: payload.client_id,
            client_secret: payload.client_secret,
            code: payload.code,
            username: payload.username,
            password: payload.password,
            refresh_token: payload.refresh_token,
            base_url,
            grant_type: payload.grant_type,
            scope: payload.scope,
        })
        .await?;

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

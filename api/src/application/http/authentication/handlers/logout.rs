use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};

use crate::application::http::server::{api_entities::api_error::ApiError, app_state::AppState};
use crate::application::url::FullUrl;

const AUTH_SESSION_COOKIE: &str = "FERRISKEY_SESSION";
const IDENTITY_COOKIE: &str = "FERRISKEY_IDENTITY";

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/logout",
    tag = "auth",
    summary = "Logout user session",
    description = "Clears Ferriskey browser session cookies used for SSO.",
    responses((status = 204, description = "Session cookies cleared"))
)]
pub async fn logout(
    State(_state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
) -> Result<impl IntoResponse, ApiError> {
    let is_secure = base_url.starts_with("https://");
    let mut clear_session_cookie = Cookie::build((AUTH_SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    let mut clear_identity_cookie = Cookie::build((IDENTITY_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if is_secure {
        clear_session_cookie = clear_session_cookie.secure(true);
        clear_identity_cookie = clear_identity_cookie.secure(true);
    }

    let clear_session_cookie_value = HeaderValue::from_str(&clear_session_cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".to_string()))?;
    let clear_identity_cookie_value = HeaderValue::from_str(&clear_identity_cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".to_string()))?;

    let mut headers = HeaderMap::new();
    headers.append(SET_COOKIE, clear_session_cookie_value);
    headers.append(SET_COOKIE, clear_identity_cookie_value);

    Ok((StatusCode::NO_CONTENT, headers))
}

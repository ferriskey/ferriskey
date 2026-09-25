use axum::http::{HeaderMap, HeaderValue, header::SET_COOKIE};
use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_core::domain::authentication::ports::OpenedSsoSession;

use crate::api_entities::api_error::ApiError;

pub const SSO_SESSION_COOKIE: &str = "FERRISKEY_SSO";

/// A `FERRISKEY_SSO` cookie. With no `max_age_secs` it is a browser-session
/// cookie, dropped when the browser closes.
pub fn set(
    secret: String,
    max_age_secs: Option<i64>,
    is_secure: bool,
) -> Result<HeaderValue, ApiError> {
    let mut cookie = Cookie::build((SSO_SESSION_COOKIE, secret))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if let Some(max_age_secs) = max_age_secs {
        cookie = cookie.max_age(time::Duration::seconds(max_age_secs.max(0)));
    }

    if is_secure {
        cookie = cookie.secure(true);
    }

    HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))
}

pub fn clear(is_secure: bool) -> Result<HeaderValue, ApiError> {
    let mut cookie = Cookie::build((SSO_SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .removal();

    if is_secure {
        cookie = cookie.secure(true);
    }

    HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))
}

/// The `Set-Cookie` header handing the browser the SSO session a login step
/// just opened, if it opened one.
pub fn headers(session: Option<OpenedSsoSession>, is_secure: bool) -> Result<HeaderMap, ApiError> {
    let mut headers = HeaderMap::new();

    if let Some(session) = session {
        headers.append(
            SET_COOKIE,
            set(session.cookie, session.max_age_secs, is_secure)?,
        );
    }

    Ok(headers)
}

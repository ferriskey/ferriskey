//! The browser's handle on its SSO session.
//!
//! It holds a secret naming one row of `user_sessions`, and nothing else. The
//! previous design put the access token itself in a cookie, which tied single sign-on
//! to a five minute token and only ever worked for clients that exchanged their code
//! in the browser.
//!
//! The secret is not the session id: that id is handed out by the session listing
//! API to any holder of `view_users`, so using it as the cookie would turn a
//! read-only permission into impersonation.

use axum::http::HeaderValue;
use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_api_core::api_entities::api_error::ApiError;

pub const SSO_SESSION_COOKIE: &str = "FERRISKEY_SSO";

/// The `Set-Cookie` value handing an SSO session to the browser.
///
/// `max_age` comes from the session itself, so the cookie dies with it instead of
/// with the browser window.
pub fn set(secret: String, max_age_secs: i64, is_secure: bool) -> Result<HeaderValue, ApiError> {
    let mut cookie = Cookie::build((SSO_SESSION_COOKIE, secret))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(max_age_secs.max(0)));

    if is_secure {
        cookie = cookie.secure(true);
    }

    HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))
}

/// The `Set-Cookie` value removing it.
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

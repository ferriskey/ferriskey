use axum::extract::{FromRequestParts, Path};
use axum::http::{HeaderMap, HeaderValue, header::SET_COOKIE, request::Parts};
use axum_cookie::CookieManager;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_core::domain::authentication::ports::AccountSelectionService;
use tracing::warn;

use crate::api_entities::api_error::ApiError;
use crate::app_state::AppState;

pub const DEVICE_COOKIE: &str = "FERRISKEY_DEVICE";

pub fn set(token: String, max_age_secs: i64, is_secure: bool) -> Result<HeaderValue, ApiError> {
    let mut cookie = Cookie::build((DEVICE_COOKIE, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if is_secure {
        cookie = cookie.secure(true);
    }

    HeaderValue::from_str(&format!("{}; Max-Age={}", cookie, max_age_secs.max(0)))
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))
}

pub fn device_token(cookie: &CookieManager) -> Option<String> {
    cookie
        .get(DEVICE_COOKIE)
        .map(|cookie| cookie.value().trim().to_string())
        .filter(|value| !value.is_empty())
}

pub struct LoginDevice {
    pub realm_name: String,
    pub token: Option<String>,
}

impl<S: Send + Sync> FromRequestParts<S> for LoginDevice {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(realm_name) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::BadRequest("Invalid realm in path".into()))?;
        let token = CookieJar::from_headers(&parts.headers)
            .get(DEVICE_COOKIE)
            .map(|cookie| cookie.value().trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self { realm_name, token })
    }
}

pub async fn attach_login(
    state: &AppState,
    realm_name: &str,
    existing: Option<String>,
    sso_secret: &str,
    is_secure: bool,
    headers: &mut HeaderMap,
) {
    let attached = match state
        .service
        .attach_login_to_device(
            realm_name.to_string(),
            existing.clone(),
            sso_secret.to_string(),
        )
        .await
    {
        Ok(Some(attached)) => attached,
        Ok(None) => return,
        Err(e) => {
            warn!(realm = %realm_name, error = %e, "Failed to attach the session to the device");
            return;
        }
    };

    let Some(token) = attached.token.or(existing) else {
        return;
    };

    match set(token, attached.max_age_secs, is_secure) {
        Ok(value) => {
            headers.append(SET_COOKIE, value);
        }
        Err(e) => warn!(realm = %realm_name, error = ?e, "Failed to build the device cookie"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_device_cookie_is_http_only_lax_and_bounded() {
        let header = set("abc".into(), 3600, false).expect("a valid header");
        let value = header.to_str().expect("ascii");

        assert!(value.starts_with("FERRISKEY_DEVICE=abc"));
        assert!(value.contains("HttpOnly"));
        assert!(value.contains("SameSite=Lax"));
        assert!(value.contains("Max-Age=3600"));
        assert!(value.contains("Path=/"));
        assert!(!value.contains("Secure"));
    }

    #[test]
    fn the_device_cookie_is_secure_over_https() {
        let header = set("abc".into(), 60, true).expect("a valid header");

        assert!(header.to_str().expect("ascii").contains("Secure"));
    }

    #[test]
    fn a_negative_lifetime_is_clamped_to_zero() {
        let header = set("abc".into(), -5, false).expect("a valid header");

        assert!(header.to_str().expect("ascii").contains("Max-Age=0"));
    }
}

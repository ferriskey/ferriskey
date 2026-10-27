use axum::{
    Extension,
    http::{HeaderMap, HeaderValue},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_core::domain::{trident::ports::LoginProgress, user::entities::RequiredAction};

use crate::{api_entities::api_error::ApiError, sso_cookie};

pub const LOGIN_ACTION_COOKIE: &str = "FERRISKEY_LOGIN_ACTION";

/// What the login-action middleware read from the step token: the login it
/// belongs to, and whether that login already has its second factor.
#[derive(Debug, Clone, Copy)]
pub struct LoginActionStep {
    pub auth_session_id: uuid::Uuid,
    pub second_factor_verified: bool,
}

/// Set on the response of a login step that leaves required actions due, so the
/// login-action middleware hands the browser a step token for the next one.
#[derive(Debug, Clone, Copy)]
pub struct LoginContinues;

pub fn login_action_cookie(
    token: String,
    root_path: &str,
    realm_name: &str,
    is_secure: bool,
) -> Result<HeaderValue, ApiError> {
    let mut cookie = Cookie::build((LOGIN_ACTION_COOKIE, token))
        .path(login_action_cookie_path(root_path, realm_name))
        .http_only(true)
        .same_site(SameSite::Lax);

    if is_secure {
        cookie = cookie.secure(true);
    }

    HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))
}

fn login_action_cookie_path(root_path: &str, realm_name: &str) -> String {
    format!("{root_path}/realms/{realm_name}/login-actions")
}

/// What a successful login step sends back: the client redirect once the login
/// is over, or the required actions still due.
pub struct LoginStepReply {
    pub headers: HeaderMap,
    pub continues: Option<Extension<LoginContinues>>,
    pub url: Option<String>,
    pub required_actions: Vec<RequiredAction>,
}

impl LoginStepReply {
    pub fn new(progress: LoginProgress, is_secure: bool) -> Result<Self, ApiError> {
        match progress {
            LoginProgress::ActionsPending(required_actions) => Ok(Self {
                headers: HeaderMap::new(),
                continues: Some(Extension(LoginContinues)),
                url: None,
                required_actions,
            }),
            LoginProgress::Completed {
                login_url,
                sso_session,
            } => Ok(Self {
                headers: sso_cookie::headers(sso_session, is_secure)?,
                continues: None,
                url: Some(login_url),
                required_actions: Vec::new(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cookie_is_scoped_to_the_realm_login_actions_without_a_root_path() {
        assert_eq!(
            login_action_cookie_path("", "master"),
            "/realms/master/login-actions"
        );
    }

    #[test]
    fn the_cookie_path_carries_the_root_path_the_routes_are_mounted_under() {
        assert_eq!(
            login_action_cookie_path("/api", "master"),
            "/api/realms/master/login-actions"
        );
    }

    #[test]
    fn pending_actions_ask_for_a_follow_up_step_token() {
        let reply = LoginStepReply::new(
            LoginProgress::ActionsPending(vec![RequiredAction::ConfigurePasskey]),
            true,
        )
        .expect("no cookie to build");

        assert!(reply.continues.is_some());
        assert!(reply.url.is_none());
        assert_eq!(
            reply.required_actions,
            vec![RequiredAction::ConfigurePasskey]
        );
    }

    #[test]
    fn a_completed_login_hands_back_the_redirect_and_no_step_token() {
        let reply = LoginStepReply::new(
            LoginProgress::Completed {
                login_url: "https://app.example/cb?code=abc".to_string(),
                sso_session: None,
            },
            true,
        )
        .expect("no cookie to build");

        assert!(reply.continues.is_none());
        assert_eq!(
            reply.url.as_deref(),
            Some("https://app.example/cb?code=abc")
        );
        assert!(reply.required_actions.is_empty());
    }
}

use axum::extract::Path;
use axum::http::header::LOCATION;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use axum_cookie::CookieManager;

use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_core::domain::authentication::entities::{
    AuthInput, AuthenticateInput, AuthenticateOutput, AuthenticationStepStatus,
};
use ferriskey_core::domain::authentication::ports::AuthService;
use ferriskey_core::domain::authentication::value_objects::CodeChallengeMethod;
use ferriskey_core::domain::common::entities::app_errors::CoreError;
use serde::{Deserialize, Serialize};
use tracing::warn;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use ferriskey_api_core::request_context::RequestContext;
use ferriskey_api_core::url::FullUrl;
pub use ferriskey_api_core::url::root_scoped_base_url;
use ferriskey_api_core::{api_entities::api_error::ApiError, app_state::AppState};

use crate::handlers::authorization_server_metadata::realm_issuer;
use crate::sso_cookie::SSO_SESSION_COOKIE;

const AUTH_SESSION_COOKIE: &str = "FERRISKEY_SESSION";
const IDENTITY_COOKIE: &str = "FERRISKEY_IDENTITY";

const SESSION_EXPIRED_MARKER: &str = "session_expired=1";

fn mark_session_expired(login_url: &str) -> String {
    if login_url.contains(SESSION_EXPIRED_MARKER) {
        return login_url.to_string();
    }

    let separator = if login_url.contains('?') { '&' } else { '?' };

    format!("{login_url}{separator}{SESSION_EXPIRED_MARKER}")
}

fn webapp_login_url(webapp_url: &str, realm_name: &str, login_url: &str) -> String {
    format!(
        "{}/realms/{}/authentication/login{}",
        webapp_url.trim_end_matches('/'),
        realm_name,
        login_url
    )
}

fn sso_success_response(
    auth_result: ferriskey_core::domain::authentication::entities::AuthenticateOutput,
    is_secure: bool,
) -> Result<axum::response::Response, ApiError> {
    let redirect_url = auth_result
        .redirect_url
        .ok_or_else(|| ApiError::InternalServerError("Missing redirect".into()))?;

    let mut response = axum::response::Response::builder()
        .status(StatusCode::FOUND)
        .header(LOCATION, &redirect_url);

    if let Some(secret) = auth_result.sso_cookie {
        response = response.header(
            SET_COOKIE,
            crate::sso_cookie::set(secret, auth_result.sso_session_max_age_secs, is_secure)?,
        );
    }

    Ok(response
        .body(axum::body::Body::empty())
        .map_err(|_| ApiError::InternalServerError("Failed to build response".into()))?
        .into_response())
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AuthRequest {
    #[validate(length(min = 1, message = "response_type is required"))]
    #[serde(default)]
    pub response_type: String,
    #[validate(length(min = 1, message = "client_id is required"))]
    #[serde(default)]
    pub client_id: String,
    #[validate(length(min = 1, message = "redirect_uri is required"))]
    #[serde(default)]
    pub redirect_uri: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub code_challenge: Option<String>,
    #[serde(default)]
    pub code_challenge_method: Option<CodeChallengeMethod>,
    /// OIDC `prompt`: `none` answers from the SSO session or fails with
    /// `login_required`, `login` always asks the user to authenticate again.
    #[serde(default)]
    pub prompt: Option<String>,
    /// OIDC `max_age`, in seconds: an SSO session authenticated longer ago
    /// than this asks the user to authenticate again.
    #[serde(default)]
    pub max_age: Option<i64>,
    /// RFC 8707 resource indicator: becomes the audience of the access token.
    /// Must be one of the realm's allowed resources.
    #[serde(default)]
    pub resource: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Prompt {
    none: bool,
    login: bool,
    consent: bool,
}

impl Prompt {
    fn parse(prompt: Option<&str>) -> Option<Self> {
        let values: Vec<&str> = prompt.unwrap_or_default().split_whitespace().collect();
        let none = values.contains(&"none");

        if none && values.len() > 1 {
            return None;
        }

        Some(Self {
            none,
            login: values.contains(&"login"),
            consent: values.contains(&"consent"),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SsoFastPathOutcome {
    Redirect,
    ConsentRequired,
    NotApplicable,
}

fn sso_fast_path_outcome(result: &AuthenticateOutput, prompt_none: bool) -> SsoFastPathOutcome {
    let is_success =
        result.status == AuthenticationStepStatus::Success && result.redirect_url.is_some();

    if !is_success {
        return SsoFastPathOutcome::NotApplicable;
    }

    if prompt_none && result.completion.is_none() {
        return SsoFastPathOutcome::ConsentRequired;
    }

    SsoFastPathOutcome::Redirect
}

/// An authorization error handed back to the client on its (already
/// validated) redirect URI, as RFC 6749 §4.1.2.1 describes.
/// It carries the issuer as `iss` (RFC 9207).
fn authorization_error_response(
    redirect_uri: &str,
    error: &str,
    state: Option<&str>,
    issuer: &str,
) -> axum::response::Response {
    let separator = if redirect_uri.contains('?') { '&' } else { '?' };
    let mut location = format!("{redirect_uri}{separator}error={error}");

    if let Some(state) = state {
        location.push_str(&format!("&state={}", urlencoding::encode(state)));
    }

    location.push_str(&format!("&iss={}", urlencoding::encode(issuer)));

    (StatusCode::FOUND, [(LOCATION, location)]).into_response()
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema, PartialEq, Eq)]
pub struct AuthResponse {
    pub url: String,
}

#[utoipa::path(
    get,
    path = "/protocol/openid-connect/auth",
    tag = "auth",
    summary = "Authenticate a user",
    description = "Initiates the authentication process for a user in a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        AuthRequest
    ),
    responses(
        (status = 302, description = "Redirects to the login page with session cookie set", body = AuthResponse),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn auth_handler(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    context: RequestContext,
    cookie: CookieManager,
    Query(params): Query<AuthRequest>,
) -> Result<axum::response::Response, ApiError> {
    let prompt = Prompt::parse(params.prompt.as_deref());
    let issuer = realm_issuer(&base_url, &state.args.server.root_path, &realm_name);

    let result = match state
        .service
        .auth(AuthInput {
            client_id: params.client_id.clone(),
            realm_name: realm_name.clone(),
            redirect_uri: params.redirect_uri.clone(),
            response_type: params.response_type.clone(),
            scope: params.scope.clone(),
            state: params.state.clone(),
            nonce: params.nonce.clone(),
            code_challenge: params.code_challenge.clone(),
            code_challenge_method: params.code_challenge_method.clone(),
            ip_address: context.ip_address,
            user_agent: context.user_agent,
            prompt_consent: prompt.as_ref().is_some_and(|prompt| prompt.consent),
            issuer: Some(issuer.clone()),
            resource: params.resource.clone(),
        })
        .await
    {
        Ok(result) => result,
        // For these errors we must NOT redirect to redirect_uri (it may be invalid / unknown).
        // Instead, redirect the browser to the FerrisKey login page with a human-readable
        // error message so the user sees a proper UI rather than raw JSON.
        Err(
            e @ CoreError::InvalidRedirectUri
            | e @ CoreError::ClientNotFound
            | e @ CoreError::InvalidRealm,
        ) => {
            warn!(
                realm = %realm_name,
                client_id = %params.client_id,
                error = %e,
                "Auth flow rejected — redirecting to login error page"
            );
            let error_url = format!(
                "{}/realms/{}/authentication/login?login_error={}",
                state.args.webapp_url.trim_end_matches('/'),
                realm_name,
                urlencoding::encode(&e.to_string()),
            );
            return Ok((StatusCode::FOUND, [(LOCATION, error_url)]).into_response());
        }
        Err(CoreError::InvalidTarget) => {
            return Ok(authorization_error_response(
                &params.redirect_uri,
                "invalid_target",
                params.state.as_deref(),
                &issuer,
            ));
        }
        Err(e) => return Err(ApiError::from(e)),
    };

    let Some(prompt) = prompt else {
        return Ok(authorization_error_response(
            &params.redirect_uri,
            "invalid_request",
            params.state.as_deref(),
            &issuer,
        ));
    };

    let is_secure = base_url.starts_with("https://");
    let flow_base_url = root_scoped_base_url(&base_url, &state.args.server.root_path);

    // `max_age=0` asks for a fresh authentication, like `prompt=login`.
    let reauthenticate = prompt.login || params.max_age == Some(0);

    let sso_cookie = cookie
        .get(SSO_SESSION_COOKIE)
        .map(|c| c.value().trim().to_string())
        .filter(|value| !value.is_empty());

    let mut sso_refusal = None;

    if let Some(sso_cookie) = sso_cookie.clone().filter(|_| !reauthenticate) {
        let auth_result = state
            .service
            .authenticate(AuthenticateInput::with_sso_session(
                realm_name.clone(),
                params.client_id.clone(),
                result.session.id,
                flow_base_url,
                sso_cookie,
                params.max_age,
            ))
            .await;

        match auth_result {
            Ok(auth_result) => match sso_fast_path_outcome(&auth_result, prompt.none) {
                SsoFastPathOutcome::Redirect => {
                    return sso_success_response(auth_result, is_secure);
                }
                SsoFastPathOutcome::ConsentRequired => {
                    return Ok(authorization_error_response(
                        &params.redirect_uri,
                        "consent_required",
                        params.state.as_deref(),
                        &issuer,
                    ));
                }
                SsoFastPathOutcome::NotApplicable => {}
            },
            Err(e) => {
                warn!(
                    realm = %realm_name,
                    client_id = %params.client_id,
                    session_code = %result.session.id,
                    error = ?e,
                    "SSO session refused, falling back to the login page"
                );
                sso_refusal = Some(e);
            }
        }
    }

    // `prompt=none` never shows a login page: the client learns why instead.
    if prompt.none {
        let error = match sso_refusal {
            Some(CoreError::Forbidden(_)) => "interaction_required",
            _ => "login_required",
        };

        return Ok(authorization_error_response(
            &params.redirect_uri,
            error,
            params.state.as_deref(),
            &issuer,
        ));
    }

    // A cookie still naming a live session (prompt=login, max_age exceeded, a
    // step still due) is kept: the login re-authenticates into its session,
    // and an abandoned login leaves the other applications signed in.
    let keeps_live_session = match sso_cookie {
        Some(sso_cookie) => state
            .service
            .remember_reauthentication(realm_name.clone(), result.session.id, sso_cookie)
            .await
            .unwrap_or_else(|e| {
                warn!(error = ?e, "Failed to remember the session to re-authenticate into");
                false
            }),
        None => false,
    };

    let mut full_url = webapp_login_url(&state.args.webapp_url, &realm_name, &result.login_url);

    let stale_sso_cookie = cookie.get(SSO_SESSION_COOKIE).is_some() && !keeps_live_session;
    // No longer honoured since the SSO session replaced it; cleared from the
    // browsers that still carry it. Drop this after one release.
    let identity_cookie_is_stale = cookie.get(IDENTITY_COOKIE).is_some();

    if identity_cookie_is_stale || stale_sso_cookie {
        full_url = mark_session_expired(&full_url);
    }

    let mut session_cookie = Cookie::build((AUTH_SESSION_COOKIE, result.session.id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if full_url.starts_with("https") {
        session_cookie = session_cookie.secure(true)
    }

    let session_cookie_value = HeaderValue::from_str(&session_cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))?;

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, session_cookie_value);

    // Force a fresh login if an existing identity cookie did not result in SSO.
    if stale_sso_cookie {
        headers.append(SET_COOKIE, crate::sso_cookie::clear(is_secure)?);
    }

    if identity_cookie_is_stale {
        let mut clear_identity_cookie = Cookie::build((IDENTITY_COOKIE, ""))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .removal();

        if full_url.starts_with("https") {
            clear_identity_cookie = clear_identity_cookie.secure(true);
        }

        let clear_identity_cookie_value = HeaderValue::from_str(&clear_identity_cookie.to_string())
            .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))?;
        headers.append(SET_COOKIE, clear_identity_cookie_value);
    }

    let mut response_builder = axum::response::Response::builder();
    response_builder = response_builder
        .status(StatusCode::FOUND)
        .header(LOCATION, &full_url);

    for value in headers.get_all(SET_COOKIE).iter() {
        response_builder = response_builder.header(SET_COOKIE, value);
    }

    let axum_response = response_builder
        .body(axum::body::Body::empty())
        .map_err(|_| ApiError::InternalServerError("Failed to build response".into()))?;

    Ok(axum_response.into_response())
}

#[cfg(test)]
mod tests {
    use super::{
        Prompt, SsoFastPathOutcome, authorization_error_response, mark_session_expired,
        sso_fast_path_outcome, webapp_login_url,
    };
    use axum::http::{StatusCode, header::LOCATION};
    use ferriskey_core::domain::authentication::entities::{
        AuthCompletion, AuthenticateOutput, AuthenticationStepStatus,
    };
    use uuid::Uuid;

    fn output(status: AuthenticationStepStatus, redirect_url: Option<&str>) -> AuthenticateOutput {
        AuthenticateOutput {
            user_id: Uuid::new_v4(),
            status,
            authorization_code: None,
            temporary_token: None,
            required_actions: Vec::new(),
            redirect_url: redirect_url.map(str::to_string),
            completion: None,
            session_state: None,
            email: None,
            sso_cookie: None,
            sso_session_max_age_secs: None,
        }
    }

    #[test]
    fn a_login_url_that_already_carries_parameters_gains_the_marker_as_another_one() {
        assert_eq!(
            mark_session_expired(
                "https://auth.example.com/realms/master/authentication/login?client_id=mestier"
            ),
            "https://auth.example.com/realms/master/authentication/login?client_id=mestier&session_expired=1"
        );
    }

    #[test]
    fn a_login_url_without_parameters_opens_its_query_string() {
        assert_eq!(
            mark_session_expired("https://auth.example.com/realms/master/authentication/login"),
            "https://auth.example.com/realms/master/authentication/login?session_expired=1"
        );
    }

    #[test]
    fn the_marker_is_not_repeated_when_the_url_already_carries_it() {
        let marked = mark_session_expired("https://auth.example.com/login?session_expired=1");

        assert_eq!(marked, "https://auth.example.com/login?session_expired=1");
    }

    #[test]
    fn joins_webapp_login_url_without_double_slashes() {
        let full_url = webapp_login_url(
            "https://login.example.com/",
            "demo",
            "?client_id=test-client&redirect_uri=https://app.example.com/callback&state=test",
        );

        assert_eq!(
            full_url,
            "https://login.example.com/realms/demo/authentication/login?client_id=test-client&redirect_uri=https://app.example.com/callback&state=test"
        );
    }

    #[test]
    fn preserves_login_url_when_webapp_has_no_trailing_slash() {
        let full_url = webapp_login_url(
            "https://login.example.com",
            "demo",
            "?client_id=test-client",
        );

        assert_eq!(
            full_url,
            "https://login.example.com/realms/demo/authentication/login?client_id=test-client"
        );
    }

    #[test]
    fn prompt_values_are_read_from_a_space_separated_list() {
        assert_eq!(Prompt::parse(None), Some(Prompt::default()));
        assert_eq!(
            Prompt::parse(Some("none")),
            Some(Prompt {
                none: true,
                login: false,
                consent: false,
            })
        );
        assert_eq!(
            Prompt::parse(Some("consent login")),
            Some(Prompt {
                none: false,
                login: true,
                consent: true,
            })
        );
    }

    #[test]
    fn prompt_none_cannot_be_combined_with_another_value() {
        assert_eq!(Prompt::parse(Some("none login")), None);
        assert_eq!(Prompt::parse(Some("consent none")), None);
    }

    #[test]
    fn an_authorization_error_goes_back_to_the_client_with_its_state() {
        let response = authorization_error_response(
            "https://app.example/cb?tenant=acme",
            "login_required",
            Some("a b"),
            "https://auth.example/realms/demo",
        );

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response
                .headers()
                .get(LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some(
                "https://app.example/cb?tenant=acme&error=login_required&state=a%20b&iss=https%3A%2F%2Fauth.example%2Frealms%2Fdemo"
            )
        );
    }

    #[test]
    fn a_normal_sso_success_is_redirected_even_under_prompt_none() {
        let mut result = output(AuthenticationStepStatus::Success, Some("https://client/cb"));
        result.completion = Some(AuthCompletion::Redirect {
            url: "https://client/cb".to_string(),
        });

        assert_eq!(
            sso_fast_path_outcome(&result, true),
            SsoFastPathOutcome::Redirect
        );
    }

    #[test]
    fn prompt_none_blocks_a_pending_consent_redirect_with_consent_required() {
        let result = output(
            AuthenticationStepStatus::Success,
            Some("https://webapp/realms/demo/authentication/consent?consent_token=abc"),
        );

        assert_eq!(
            sso_fast_path_outcome(&result, true),
            SsoFastPathOutcome::ConsentRequired
        );
    }

    #[test]
    fn a_pending_consent_redirect_is_followed_when_prompt_is_not_none() {
        let result = output(
            AuthenticationStepStatus::Success,
            Some("https://webapp/realms/demo/authentication/consent?consent_token=abc"),
        );

        assert_eq!(
            sso_fast_path_outcome(&result, false),
            SsoFastPathOutcome::Redirect
        );
    }

    #[test]
    fn a_non_success_status_is_never_a_fast_path_outcome() {
        let result = output(AuthenticationStepStatus::RequiresActions, None);

        assert_eq!(
            sso_fast_path_outcome(&result, true),
            SsoFastPathOutcome::NotApplicable
        );
    }

    #[test]
    fn a_completed_redirect_is_distinguishable_from_a_pending_consent_one() {
        let mut result = output(AuthenticationStepStatus::Success, Some("https://client/cb"));
        result.completion = Some(AuthCompletion::Redirect {
            url: "https://client/cb".to_string(),
        });

        assert_eq!(
            sso_fast_path_outcome(&result, true),
            SsoFastPathOutcome::Redirect
        );
    }
}

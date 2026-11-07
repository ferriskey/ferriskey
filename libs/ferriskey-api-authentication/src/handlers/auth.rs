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
use ferriskey_core::domain::authentication::ports::{
    AccountSelectionService, AuthService, AuthorizeAccounts, ListAccountsInput, SelectAccountInput,
    SelectAccountOutcome,
};
use ferriskey_core::domain::authentication::value_objects::CodeChallengeMethod;
use ferriskey_core::domain::common::entities::app_errors::CoreError;
use ferriskey_core::domain::device_session::entities::AccountEntry;
use serde::{Deserialize, Serialize};
use tracing::warn;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use ferriskey_api_core::device_cookie::device_token;
use ferriskey_api_core::request_context::RequestContext;
pub use ferriskey_api_core::url::root_scoped_base_url;
use ferriskey_api_core::url::{FullUrl, WebappUrl};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryPage {
    Login,
    Register,
    AccountChooser,
}

impl EntryPage {
    fn for_prompt(prompt: &Prompt, registration_enabled: bool) -> Self {
        if prompt.create && registration_enabled {
            Self::Register
        } else {
            Self::Login
        }
    }

    fn segment(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Register => "register",
            Self::AccountChooser => "account-chooser",
        }
    }
}

fn webapp_entry_url(
    webapp_url: &str,
    realm_name: &str,
    page: EntryPage,
    login_url: &str,
) -> String {
    format!(
        "{}/realms/{}/authentication/{}{}",
        webapp_url.trim_end_matches('/'),
        realm_name,
        page.segment(),
        login_url
    )
}

fn auth_session_cookie(session_code: Uuid, is_secure: bool) -> Result<HeaderValue, ApiError> {
    let mut cookie = Cookie::build((AUTH_SESSION_COOKIE, session_code.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if is_secure {
        cookie = cookie.secure(true);
    }

    HeaderValue::from_str(&cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))
}

pub(crate) fn sso_success_response(
    auth_result: ferriskey_core::domain::authentication::entities::AuthenticateOutput,
    session_code: Uuid,
    is_secure: bool,
) -> Result<axum::response::Response, ApiError> {
    let redirect_url = auth_result
        .redirect_url
        .ok_or_else(|| ApiError::InternalServerError("Missing redirect".into()))?;

    let mut response = axum::response::Response::builder()
        .status(StatusCode::FOUND)
        .header(LOCATION, &redirect_url);

    if auth_result.completion.is_none() {
        response = response.header(SET_COOKIE, auth_session_cookie(session_code, is_secure)?);
    }

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
    #[serde(default)]
    pub login_hint: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Prompt {
    none: bool,
    login: bool,
    select_account: bool,
    consent: bool,
    create: bool,
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
            select_account: values.contains(&"select_account"),
            consent: values.contains(&"consent"),
            create: values.contains(&"create"),
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

#[derive(Debug, PartialEq, Eq)]
enum PromptNoneDecision {
    Error(&'static str),
    Resume(Uuid),
}

struct PromptNoneFacts<'a> {
    chooser_enabled: bool,
    sso_refusal: Option<&'a CoreError>,
    entries: &'a [AccountEntry],
}

fn prompt_none_decision(facts: &PromptNoneFacts<'_>) -> PromptNoneDecision {
    if !facts.chooser_enabled {
        return PromptNoneDecision::Error(match facts.sso_refusal {
            Some(CoreError::Forbidden(_)) => "interaction_required",
            _ => "login_required",
        });
    }

    match facts.entries {
        [] => PromptNoneDecision::Error(match facts.sso_refusal {
            Some(CoreError::Forbidden(_) | CoreError::ReauthenticationRequired) => {
                "interaction_required"
            }
            _ => "login_required",
        }),
        [entry] if entry.requires_reauthentication => {
            PromptNoneDecision::Error("interaction_required")
        }
        [entry] => PromptNoneDecision::Resume(entry.session_id),
        entries => entries
            .iter()
            .find(|entry| entry.is_active && !entry.requires_reauthentication)
            .map_or(
                PromptNoneDecision::Error("account_selection_required"),
                |entry| PromptNoneDecision::Resume(entry.session_id),
            ),
    }
}

fn resume_error_code(error: &CoreError) -> &'static str {
    match error {
        CoreError::AccountSelectionRequired => "account_selection_required",
        CoreError::Forbidden(_) | CoreError::ReauthenticationRequired => "interaction_required",
        _ => "login_required",
    }
}

fn with_login_hint(url: String, login_hint: Option<&str>) -> String {
    match login_hint.filter(|hint| !hint.is_empty()) {
        Some(hint) => {
            let separator = if url.contains('?') { '&' } else { '?' };
            format!("{url}{separator}login_hint={}", urlencoding::encode(hint))
        }
        None => url,
    }
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

async fn load_device_accounts(
    state: &AppState,
    realm_name: &str,
    params: &AuthRequest,
    cookie: &CookieManager,
    active_sso_token: Option<String>,
) -> AuthorizeAccounts {
    let loaded = state
        .service
        .authorize_accounts(ListAccountsInput {
            realm_name: realm_name.to_string(),
            device_token: device_token(cookie),
            active_sso_token,
            max_age: params.max_age,
            client_id: params.client_id.clone(),
            session_code: None,
        })
        .await;

    loaded.unwrap_or_else(|e| {
        warn!(realm = %realm_name, error = ?e, "Failed to list the device accounts");
        AuthorizeAccounts {
            chooser_enabled: false,
            entries: Vec::new(),
        }
    })
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
    WebappUrl(webapp_url): WebappUrl,
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
            max_age: params.max_age,
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
                webapp_url,
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
    let reauthenticate = prompt.login || prompt.create || params.max_age == Some(0);

    let sso_cookie = cookie
        .get(SSO_SESSION_COOKIE)
        .map(|c| c.value().trim().to_string())
        .filter(|value| !value.is_empty());

    let device_accounts = if prompt.select_account {
        Some(load_device_accounts(&state, &realm_name, &params, &cookie, sso_cookie.clone()).await)
    } else {
        None
    };

    let chooser_forced = device_accounts
        .as_ref()
        .is_some_and(|accounts| accounts.chooser_enabled);
    let show_chooser = device_accounts
        .as_ref()
        .is_some_and(|accounts| accounts.chooser_enabled && !accounts.entries.is_empty());

    let mut sso_refusal = None;

    if let Some(sso_cookie) = sso_cookie
        .clone()
        .filter(|_| !reauthenticate && !chooser_forced)
    {
        let auth_result = state
            .service
            .authenticate(AuthenticateInput::with_sso_session(
                realm_name.clone(),
                params.client_id.clone(),
                result.session.id,
                flow_base_url,
                webapp_url.clone(),
                sso_cookie,
                params.max_age,
            ))
            .await;

        match auth_result {
            Ok(auth_result) => match sso_fast_path_outcome(&auth_result, prompt.none) {
                SsoFastPathOutcome::Redirect => {
                    return sso_success_response(auth_result, result.session.id, is_secure);
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
        let accounts =
            load_device_accounts(&state, &realm_name, &params, &cookie, sso_cookie.clone()).await;

        let decision = prompt_none_decision(&PromptNoneFacts {
            chooser_enabled: accounts.chooser_enabled,
            sso_refusal: sso_refusal.as_ref(),
            entries: &accounts.entries,
        });

        let error = match decision {
            PromptNoneDecision::Error(error) => error,
            PromptNoneDecision::Resume(session_id) => {
                let outcome = state
                    .service
                    .select_account(SelectAccountInput {
                        realm_name: realm_name.clone(),
                        device_token: device_token(&cookie),
                        session_code: result.session.id,
                        session_id,
                        max_age: params.max_age,
                        webapp_base_url: webapp_url.clone(),
                    })
                    .await;

                match outcome {
                    Ok(SelectAccountOutcome::Resumed(output)) => {
                        match sso_fast_path_outcome(&output, true) {
                            SsoFastPathOutcome::Redirect => {
                                return sso_success_response(*output, result.session.id, is_secure);
                            }
                            SsoFastPathOutcome::ConsentRequired => "consent_required",
                            SsoFastPathOutcome::NotApplicable => "interaction_required",
                        }
                    }
                    Ok(SelectAccountOutcome::RequiresCredentials { .. }) => "interaction_required",
                    Err(e) => {
                        warn!(
                            realm = %realm_name,
                            client_id = %params.client_id,
                            error = ?e,
                            "Device account refused under prompt=none"
                        );
                        resume_error_code(&e)
                    }
                }
            }
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

    let entry_page = if show_chooser {
        EntryPage::AccountChooser
    } else {
        EntryPage::for_prompt(&prompt, result.registration_enabled)
    };

    let mut full_url = with_login_hint(
        webapp_entry_url(&webapp_url, &realm_name, entry_page, &result.login_url),
        params.login_hint.as_deref(),
    );

    let stale_sso_cookie = cookie.get(SSO_SESSION_COOKIE).is_some() && !keeps_live_session;
    // No longer honoured since the SSO session replaced it; cleared from the
    // browsers that still carry it. Drop this after one release.
    let identity_cookie_is_stale = cookie.get(IDENTITY_COOKIE).is_some();

    if identity_cookie_is_stale || stale_sso_cookie {
        full_url = mark_session_expired(&full_url);
    }

    let session_cookie_value =
        auth_session_cookie(result.session.id, full_url.starts_with("https"))?;

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
        EntryPage, Prompt, PromptNoneDecision, PromptNoneFacts, SsoFastPathOutcome,
        authorization_error_response, mark_session_expired, prompt_none_decision,
        resume_error_code, sso_fast_path_outcome, webapp_entry_url, with_login_hint,
    };
    use axum::http::{StatusCode, header::LOCATION};
    use ferriskey_core::domain::authentication::entities::{
        AuthCompletion, AuthenticateOutput, AuthenticationStepStatus,
    };
    use ferriskey_core::domain::common::entities::app_errors::CoreError;
    use ferriskey_core::domain::device_session::entities::AccountEntry;
    use ferriskey_core::domain::session::entities::SessionState;
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
        let full_url = webapp_entry_url(
            "https://login.example.com/",
            "demo",
            EntryPage::Login,
            "?client_id=test-client&redirect_uri=https://app.example.com/callback&state=test",
        );

        assert_eq!(
            full_url,
            "https://login.example.com/realms/demo/authentication/login?client_id=test-client&redirect_uri=https://app.example.com/callback&state=test"
        );
    }

    #[test]
    fn preserves_login_url_when_webapp_has_no_trailing_slash() {
        let full_url = webapp_entry_url(
            "https://login.example.com",
            "demo",
            EntryPage::Login,
            "?client_id=test-client",
        );

        assert_eq!(
            full_url,
            "https://login.example.com/realms/demo/authentication/login?client_id=test-client"
        );
    }

    #[test]
    fn the_register_page_keeps_the_same_query_as_the_login_page() {
        let full_url = webapp_entry_url(
            "https://login.example.com/",
            "demo",
            EntryPage::Register,
            "?client_id=test-client&state=s1",
        );

        assert_eq!(
            full_url,
            "https://login.example.com/realms/demo/authentication/register?client_id=test-client&state=s1"
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
                select_account: false,
                consent: false,
                create: false,
            })
        );
        assert_eq!(
            Prompt::parse(Some("consent login")),
            Some(Prompt {
                none: false,
                login: true,
                select_account: false,
                consent: true,
                create: false,
            })
        );
        assert_eq!(
            Prompt::parse(Some("create")),
            Some(Prompt {
                none: false,
                login: false,
                select_account: false,
                consent: false,
                create: true,
            })
        );
    }

    #[test]
    fn prompt_none_cannot_be_combined_with_another_value() {
        assert_eq!(Prompt::parse(Some("none login")), None);
        assert_eq!(Prompt::parse(Some("consent none")), None);
        assert_eq!(Prompt::parse(Some("none create")), None);
        assert_eq!(Prompt::parse(Some("create none")), None);
    }

    #[test]
    fn prompt_create_opens_the_register_page_when_the_realm_allows_registration() {
        let prompt = Prompt::parse(Some("create")).expect("create alone is valid");

        assert_eq!(EntryPage::for_prompt(&prompt, true), EntryPage::Register);
    }

    #[test]
    fn prompt_create_falls_back_to_the_login_page_when_registration_is_disabled() {
        let prompt = Prompt::parse(Some("create")).expect("create alone is valid");

        assert_eq!(EntryPage::for_prompt(&prompt, false), EntryPage::Login);
    }

    #[test]
    fn without_prompt_create_the_login_page_opens_whatever_the_realm_allows() {
        for value in [None, Some("login"), Some("consent"), Some("login consent")] {
            let prompt = Prompt::parse(value).expect("valid prompt");

            assert_eq!(EntryPage::for_prompt(&prompt, true), EntryPage::Login);
        }
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

    #[test]
    fn select_account_and_consent_are_parsed_alongside_the_other_values() {
        let prompt = Prompt::parse(Some("select_account consent login")).expect("valid prompt");

        assert!(prompt.select_account && prompt.consent && prompt.login);
        assert!(!prompt.none && !prompt.create);
        assert!(Prompt::parse(Some("select_account")).is_some_and(|p| p.select_account));
    }

    #[test]
    fn prompt_none_cannot_be_combined_with_select_account() {
        assert_eq!(Prompt::parse(Some("none select_account")), None);
        assert_eq!(Prompt::parse(Some("select_account none")), None);
    }

    #[test]
    fn the_account_chooser_page_keeps_the_same_query_as_the_login_page() {
        let full_url = webapp_entry_url(
            "https://login.example.com/",
            "demo",
            EntryPage::AccountChooser,
            "?client_id=test-client&state=s1",
        );

        assert_eq!(
            full_url,
            "https://login.example.com/realms/demo/authentication/account-chooser?client_id=test-client&state=s1"
        );
    }

    #[test]
    fn a_login_hint_is_appended_verbatim_and_encoded() {
        assert_eq!(
            with_login_hint("https://w/login?client_id=c".into(), Some("a+b@x.io")),
            "https://w/login?client_id=c&login_hint=a%2Bb%40x.io"
        );
        assert_eq!(
            with_login_hint("https://w/login?client_id=c".into(), None),
            "https://w/login?client_id=c"
        );
        assert_eq!(
            with_login_hint("https://w/login?client_id=c".into(), Some("")),
            "https://w/login?client_id=c"
        );
    }

    fn entry(is_active: bool, requires_reauthentication: bool) -> AccountEntry {
        AccountEntry {
            session_id: Uuid::new_v4(),
            display_name: "Alice".into(),
            masked_email: None,
            state: SessionState::Active,
            last_seen_at: None,
            is_active,
            requires_reauthentication,
        }
    }

    fn decide(
        chooser_enabled: bool,
        sso_refusal: Option<&CoreError>,
        entries: &[AccountEntry],
    ) -> PromptNoneDecision {
        prompt_none_decision(&PromptNoneFacts {
            chooser_enabled,
            sso_refusal,
            entries,
        })
    }

    #[test]
    fn with_the_chooser_off_prompt_none_keeps_the_legacy_mapping() {
        let entries = [entry(true, false), entry(false, false)];
        let forbidden = CoreError::Forbidden("owed".into());

        assert_eq!(
            decide(false, Some(&forbidden), &entries),
            PromptNoneDecision::Error("interaction_required")
        );
        assert_eq!(
            decide(false, Some(&CoreError::ReauthenticationRequired), &entries),
            PromptNoneDecision::Error("login_required")
        );
        assert_eq!(
            decide(false, None, &entries),
            PromptNoneDecision::Error("login_required")
        );
    }

    #[test]
    fn no_entry_means_login_required() {
        assert_eq!(
            decide(true, None, &[]),
            PromptNoneDecision::Error("login_required")
        );
    }

    #[test]
    fn no_entry_but_a_live_session_owing_a_step_means_interaction_required() {
        let forbidden = CoreError::Forbidden("owed".into());

        assert_eq!(
            decide(true, Some(&forbidden), &[]),
            PromptNoneDecision::Error("interaction_required")
        );
        assert_eq!(
            decide(true, Some(&CoreError::ReauthenticationRequired), &[]),
            PromptNoneDecision::Error("interaction_required")
        );
        assert_eq!(
            decide(true, Some(&CoreError::DeviceSessionNotFound), &[]),
            PromptNoneDecision::Error("login_required")
        );
    }

    #[test]
    fn a_single_resumable_entry_is_resumed() {
        let only = entry(false, false);

        assert_eq!(
            decide(true, None, std::slice::from_ref(&only)),
            PromptNoneDecision::Resume(only.session_id)
        );
    }

    #[test]
    fn a_single_entry_owing_a_step_means_interaction_required() {
        assert_eq!(
            decide(true, None, &[entry(true, true)]),
            PromptNoneDecision::Error("interaction_required")
        );
    }

    #[test]
    fn several_entries_resume_the_resumable_active_one() {
        let active = entry(true, false);
        let entries = [entry(false, false), active.clone(), entry(false, false)];

        assert_eq!(
            decide(true, None, &entries),
            PromptNoneDecision::Resume(active.session_id)
        );
    }

    #[test]
    fn several_entries_without_a_resumable_active_one_need_a_selection() {
        assert_eq!(
            decide(true, None, &[entry(false, false), entry(false, false)]),
            PromptNoneDecision::Error("account_selection_required")
        );
        assert_eq!(
            decide(true, None, &[entry(true, true), entry(false, false)]),
            PromptNoneDecision::Error("account_selection_required")
        );
    }

    #[test]
    fn a_refused_device_resume_maps_to_the_matching_error() {
        assert_eq!(
            resume_error_code(&CoreError::AccountSelectionRequired),
            "account_selection_required"
        );
        assert_eq!(
            resume_error_code(&CoreError::Forbidden("x".into())),
            "interaction_required"
        );
        assert_eq!(
            resume_error_code(&CoreError::ReauthenticationRequired),
            "interaction_required"
        );
        assert_eq!(
            resume_error_code(&CoreError::DeviceSessionNotFound),
            "login_required"
        );
    }
}

use axum::body::Body;
use axum::extract::{Form, Path, Query, State};
use axum::http::header::{LOCATION, SET_COOKIE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_api_core::api_entities::api_error::ApiError;
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::url::FullUrl;
use ferriskey_core::domain::common::entities::app_errors::CoreError;
use ferriskey_core::domain::saml::entities::StartSsoInput;
use ferriskey_core::domain::saml::ports::SamlService;
use ferriskey_saml::binding::{BindingError, decode_post, decode_redirect};
use serde::Deserialize;
use tracing::warn;
use utoipa::{IntoParams, ToSchema};

use crate::html::error_page;
use crate::origin::{saml_public_base_url, webapp_login_url};

const AUTH_SESSION_COOKIE: &str = "FERRISKEY_SESSION";

const UNREADABLE_REQUEST: &str = "The single sign-on request could not be read.";
const REFUSED_REQUEST: &str = "This single sign-on request was refused.";
const UNAVAILABLE: &str = "Single sign-on could not be started. Try again from your application.";

#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SamlAuthnRequestParams {
    #[serde(rename = "SAMLRequest")]
    pub saml_request: String,
    #[serde(rename = "RelayState", default)]
    pub relay_state: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum Binding {
    HttpRedirect,
    HttpPost,
}

impl Binding {
    fn decode(self, message: &str) -> Result<String, BindingError> {
        match self {
            Self::HttpRedirect => decode_redirect(message),
            Self::HttpPost => decode_post(message),
        }
    }
}

#[utoipa::path(
    get,
    path = "/protocol/saml",
    tag = "saml",
    summary = "Start a SAML single sign-on over the HTTP-Redirect binding",
    description = "Accepts a deflated, base64 encoded AuthnRequest from a service provider and redirects the browser to the login page.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        SamlAuthnRequestParams
    ),
    responses(
        (status = 302, description = "Redirects to the login page with the session cookie set"),
        (status = 400, description = "The AuthnRequest was unreadable or refused, rendered as HTML"),
        (status = 500, description = "Internal Server Error, rendered as HTML")
    )
)]
pub async fn saml_sso_redirect(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    Query(params): Query<SamlAuthnRequestParams>,
) -> Result<Response, ApiError> {
    begin_sso(state, realm_name, base_url, Binding::HttpRedirect, params).await
}

#[utoipa::path(
    post,
    path = "/protocol/saml",
    tag = "saml",
    summary = "Start a SAML single sign-on over the HTTP-POST binding",
    description = "Accepts a base64 encoded AuthnRequest posted as a form by a service provider and redirects the browser to the login page.",
    params(("realm_name" = String, Path, description = "Realm name")),
    request_body(content = SamlAuthnRequestParams, content_type = "application/x-www-form-urlencoded"),
    responses(
        (status = 302, description = "Redirects to the login page with the session cookie set"),
        (status = 400, description = "The AuthnRequest was unreadable or refused, rendered as HTML"),
        (status = 500, description = "Internal Server Error, rendered as HTML")
    )
)]
pub async fn saml_sso_post(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    Form(params): Form<SamlAuthnRequestParams>,
) -> Result<Response, ApiError> {
    begin_sso(state, realm_name, base_url, Binding::HttpPost, params).await
}

async fn begin_sso(
    state: AppState,
    realm_name: String,
    base_url: String,
    binding: Binding,
    params: SamlAuthnRequestParams,
) -> Result<Response, ApiError> {
    let authn_request = match binding.decode(&params.saml_request) {
        Ok(authn_request) => authn_request,
        Err(reason) => {
            warn!(
                realm = %realm_name,
                %reason,
                "rejecting a saml authn request that could not be decoded"
            );

            return Ok(error_page(StatusCode::BAD_REQUEST, UNREADABLE_REQUEST));
        }
    };

    let public_base_url = saml_public_base_url(
        state.args.server.public_url.as_deref(),
        &base_url,
        &state.args.server.root_path,
    );

    let output = match state
        .service
        .start_sso(StartSsoInput {
            realm_name: realm_name.clone(),
            authn_request,
            relay_state: params.relay_state,
            public_base_url,
        })
        .await
    {
        Ok(output) => output,
        Err(error) => return Ok(refusal(&realm_name, error)),
    };

    redirect_to_login(
        &webapp_login_url(&state.args.webapp_url, &realm_name, &output.login_url),
        &output.session.id.to_string(),
    )
}

fn refusal(realm_name: &str, error: CoreError) -> Response {
    let (status, message) = match &error {
        CoreError::InvalidRequest
        | CoreError::InvalidRealm
        | CoreError::InvalidClient
        | CoreError::ClientNotFound
        | CoreError::SamlConfigNotFound
        | CoreError::InvalidRedirectUri => (StatusCode::BAD_REQUEST, REFUSED_REQUEST),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, UNAVAILABLE),
    };

    warn!(
        realm = %realm_name,
        %error,
        "refusing a saml single sign-on request"
    );

    error_page(status, message)
}

fn redirect_to_login(login_url: &str, session_id: &str) -> Result<Response, ApiError> {
    let mut session_cookie = Cookie::build((AUTH_SESSION_COOKIE, session_id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if login_url.starts_with("https") {
        session_cookie = session_cookie.secure(true);
    }

    let session_cookie = HeaderValue::from_str(&session_cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".into()))?;

    Response::builder()
        .status(StatusCode::FOUND)
        .header(LOCATION, login_url)
        .header(SET_COOKIE, session_cookie)
        .body(Body::empty())
        .map_err(|_| ApiError::InternalServerError("Failed to build response".into()))
}

#[cfg(test)]
mod tests {
    use super::{Binding, redirect_to_login, refusal};

    use axum::http::StatusCode;
    use axum::http::header::{LOCATION, SET_COOKIE};
    use ferriskey_core::domain::common::entities::app_errors::CoreError;
    use ferriskey_saml::binding::{encode_post, encode_redirect};

    const AUTHN_REQUEST: &str = r#"<samlp:AuthnRequest ID="_1"/>"#;

    fn cookie_of(response: &axum::response::Response) -> String {
        response
            .headers()
            .get(SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .expect("a session cookie must be set")
            .to_string()
    }

    #[test]
    fn the_redirect_binding_inflates_the_message_the_service_provider_deflated() {
        let encoded = encode_redirect(AUTHN_REQUEST).expect("the fixture must encode");

        assert_eq!(
            Binding::HttpRedirect
                .decode(&encoded)
                .expect("the redirect binding must inflate its own output"),
            AUTHN_REQUEST
        );
    }

    #[test]
    fn the_post_binding_reads_plain_base64() {
        assert_eq!(
            Binding::HttpPost
                .decode(&encode_post(AUTHN_REQUEST))
                .expect("the post binding must decode its own output"),
            AUTHN_REQUEST
        );
    }

    #[test]
    fn each_binding_refuses_the_encoding_of_the_other() {
        assert!(
            Binding::HttpRedirect
                .decode(&encode_post(AUTHN_REQUEST))
                .is_err()
        );
        assert!(
            Binding::HttpPost
                .decode(&encode_redirect(AUTHN_REQUEST).expect("the fixture must encode"))
                .is_err(),
            "an inflated document is not valid xml, so it must not reach the parser as one"
        );
    }

    #[test]
    fn a_request_the_realm_refuses_is_a_client_error_and_never_names_the_reason() {
        let response = refusal("master", CoreError::SamlConfigNotFound);

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn a_failure_that_is_ours_is_reported_as_ours() {
        assert_eq!(
            refusal("master", CoreError::SessionCreateError).status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn the_session_cookie_is_locked_down_exactly_as_the_openid_connect_flow_locks_it() {
        let response = redirect_to_login(
            "https://login.example.com/realms/demo/authentication/login?client_id=sp",
            "8b4d1a3e-0000-4000-8000-000000000000",
        )
        .expect("the redirect must build");

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response
                .headers()
                .get(LOCATION)
                .and_then(|value| value.to_str().ok()),
            Some("https://login.example.com/realms/demo/authentication/login?client_id=sp")
        );

        let cookie = cookie_of(&response);
        assert!(cookie.starts_with("FERRISKEY_SESSION=8b4d1a3e-0000-4000-8000-000000000000"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(cookie.contains("Secure"));
    }

    #[test]
    fn the_cookie_drops_the_secure_attribute_over_plain_http_so_local_development_still_works() {
        let response = redirect_to_login(
            "http://localhost:5555/realms/demo/authentication/login?client_id=sp",
            "8b4d1a3e-0000-4000-8000-000000000000",
        )
        .expect("the redirect must build");

        assert!(!cookie_of(&response).contains("Secure"));
    }
}

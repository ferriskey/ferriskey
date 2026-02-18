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
    AuthInput, AuthenticateInput, AuthenticationStepStatus,
};
use ferriskey_core::domain::authentication::ports::AuthService;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::application::http::server::{api_entities::api_error::ApiError, app_state::AppState};
use crate::application::url::FullUrl;

const AUTH_SESSION_COOKIE: &str = "FERRISKEY_SESSION";
const IDENTITY_COOKIE: &str = "FERRISKEY_IDENTITY";

pub fn root_scoped_base_url(base_url: &str, root_path: &str) -> String {
    if root_path.is_empty() || root_path == "/" {
        return base_url.to_string();
    }
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        root_path.trim_start_matches('/')
    )
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
    cookie: CookieManager,
    Query(params): Query<AuthRequest>,
) -> Result<axum::response::Response, ApiError> {
    let result = state
        .service
        .auth(AuthInput {
            client_id: params.client_id.clone(),
            realm_name: realm_name.clone(),
            redirect_uri: params.redirect_uri.clone(),
            response_type: params.response_type.clone(),
            scope: params.scope.clone(),
            state: params.state.clone(),
        })
        .await
        .map_err(ApiError::from)?;

    if let Some(identity_cookie) = cookie.get(IDENTITY_COOKIE) {
        let auth_result = state
            .service
            .authenticate(AuthenticateInput::with_existing_token(
                realm_name.clone(),
                params.client_id.clone(),
                result.session.id,
                root_scoped_base_url(&base_url, &state.args.server.root_path),
                identity_cookie.value().to_string(),
            ))
            .await;

        if let Ok(auth_result) = auth_result
            && auth_result.status == AuthenticationStepStatus::Success
            && let Some(redirect_url) = auth_result.redirect_url
        {
            return Ok((StatusCode::FOUND, [(LOCATION, redirect_url)]).into_response());
        }
    }

    let full_url = format!(
        "{}/realms/{}/authentication/login{}",
        state.args.webapp_url.clone(),
        realm_name,
        result.login_url.clone()
    );

    let mut session_cookie = Cookie::build((AUTH_SESSION_COOKIE, result.session.id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if full_url.starts_with("https") {
        session_cookie = session_cookie.secure(true)
    }

    let session_cookie_value = HeaderValue::from_str(&session_cookie.to_string())
        .map_err(|_| ApiError::InternalServerError("Invalid cookie header".to_string()))?;

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, session_cookie_value);

    // Force a fresh login if an existing identity cookie did not result in SSO.
    if cookie.get(IDENTITY_COOKIE).is_some() {
        let mut clear_identity_cookie = Cookie::build((IDENTITY_COOKIE, ""))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax);

        if full_url.starts_with("https") {
            clear_identity_cookie = clear_identity_cookie.secure(true);
        }

        let clear_identity_cookie_value = HeaderValue::from_str(&clear_identity_cookie.to_string())
            .map_err(|_| ApiError::InternalServerError("Invalid cookie header".to_string()))?;
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
        .map_err(|_| ApiError::InternalServerError("Failed to build response".to_string()))?;

    Ok(axum_response.into_response())
}

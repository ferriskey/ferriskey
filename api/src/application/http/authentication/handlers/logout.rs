use axum::{
    Form,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_core::domain::authentication::{ports::AuthService, value_objects::EndSessionInput};
use validator::Validate;

use crate::application::http::authentication::validators::LogoutRequestValidator;
use crate::application::http::server::{api_entities::api_error::ApiError, app_state::AppState};
use crate::application::http::{
    authentication::handlers::auth::root_scoped_base_url,
    server::api_entities::api_error::ApiErrorResponse,
};
use crate::application::url::FullUrl;

const AUTH_SESSION_COOKIE: &str = "FERRISKEY_SESSION";
const IDENTITY_COOKIE: &str = "FERRISKEY_IDENTITY";

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/logout",
    tag = "auth",
    summary = "Clear user session",
    description = "Clears Ferriskey browser session cookies used for SSO.",
    responses(
        (status = 204, description = "Session cookies cleared"),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
fn clear_session_cookies_headers(base_url: &str) -> Result<HeaderMap, ApiError> {
    let is_secure = base_url.starts_with("https://");
    let mut clear_session_cookie = Cookie::build((AUTH_SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .removal();

    let mut clear_identity_cookie = Cookie::build((IDENTITY_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .removal();

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

    Ok(headers)
}

async fn handle_logout_request(
    state: AppState,
    realm_name: String,
    base_url: String,
    payload: LogoutRequestValidator,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;

    let expected_issuer = format!(
        "{}/realms/{}",
        root_scoped_base_url(&base_url, &state.args.server.root_path),
        realm_name
    );

    let end_session = state
        .service
        .end_session(EndSessionInput {
            realm_name,
            expected_issuer,
            id_token_hint: payload.id_token_hint,
            post_logout_redirect_uri: payload.post_logout_redirect_uri,
            state: payload.state,
            client_id: payload.client_id,
        })
        .await?;

    let headers = clear_session_cookies_headers(&base_url)?;

    if let Some(redirect_uri) = end_session.redirect_uri {
        let mut response = Redirect::temporary(&redirect_uri).into_response();

        for cookie in headers.get_all(SET_COOKIE).iter() {
            response.headers_mut().append(SET_COOKIE, cookie.clone());
        }

        return Ok(response);
    }

    Ok((StatusCode::NO_CONTENT, headers).into_response())
}

#[utoipa::path(
    get,
    path = "/protocol/openid-connect/logout",
    tag = "auth",
    summary = "OIDC RP-Initiated Logout",
    description = "Ends the user's OP session. Supports id_token_hint, post_logout_redirect_uri, state, and client_id.",
    params(
      ("realm_name" = String, Path, description = "Realm name"),
      LogoutRequestValidator
    ),
    responses(
        (status = 204, description = "Session cookies cleared"),
        (status = 307, description = "Redirect to post_logout_redirect_uri")
    )
)]
pub async fn logout_get(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    Query(payload): Query<LogoutRequestValidator>,
) -> Result<impl IntoResponse, ApiError> {
    handle_logout_request(state, realm_name, base_url, payload).await
}

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/logout",
    tag = "auth",
    summary = "OIDC RP-Initiated Logout",
    description = "Ends the user's OP session. Supports id_token_hint, post_logout_redirect_uri, state, and client_id.",
    request_body = LogoutRequestValidator,
    params(
      ("realm_name" = String, Path, description = "Realm name")
    ),
    responses(
        (status = 204, description = "Session cookies cleared"),
        (status = 307, description = "Redirect to post_logout_redirect_uri")
    )
)]
pub async fn logout_post(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    Form(payload): Form<LogoutRequestValidator>,
) -> Result<impl IntoResponse, ApiError> {
    handle_logout_request(state, realm_name, base_url, payload).await
}

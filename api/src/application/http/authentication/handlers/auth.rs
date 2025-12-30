use axum::extract::Path;
use axum::http::header::LOCATION;
use axum::{
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};

use axum_extra::extract::cookie::{Cookie, SameSite};
use ferriskey_core::domain::authentication::entities::AuthInput;
use ferriskey_core::domain::authentication::ports::AuthService;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::application::http::server::{api_entities::api_error::ApiError, app_state::AppState};

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
    Query(params): Query<AuthRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .service
        .auth(AuthInput {
            client_id: params.client_id,
            realm_name: realm_name.clone(),
            redirect_uri: params.redirect_uri,
            response_type: params.response_type,
            scope: params.scope,
            state: params.state,
        })
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let full_url = format!(
        "{}/realms/{}/authentication/login{}",
        state.args.webapp_url.clone(),
        realm_name,
        result.login_url.clone()
    );

    let mut session_cookie = Cookie::build(("FERRISKEY_SESSION", result.session.id.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    if full_url.starts_with("https") {
        session_cookie = session_cookie.secure(true)
    }

    let mut axum_response = axum::response::Response::builder()
        .status(StatusCode::FOUND)
        .header(LOCATION, &full_url)
        .body(axum::body::Body::empty())
        .map_err(|_| ApiError::InternalServerError("Failed to build response".to_string()))?;

    axum_response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&session_cookie.to_string()).unwrap(),
    );

    Ok(axum_response)
}

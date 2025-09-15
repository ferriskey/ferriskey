use axum::extract::Path;
use axum::http::header::{LOCATION, ORIGIN};
use axum::{
    extract::{Query, State},
    http::{
        self, HeaderMap, HeaderValue, StatusCode,
        header::{CONTENT_TYPE, SET_COOKIE},
    },
    response::IntoResponse,
};

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
pub async fn auth(
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

    let cookie_value = format!(
        "session_code={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=3600",
        result.session.id
    );

    let session_cookie = format!(
        "FERRISKEY_SESSION={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=3600",
        result.session.id
    );

    let mut headers = HeaderMap::new();

    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie_value)
            .map_err(|_| ApiError::InternalServerError("".to_string()))?,
    );

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&session_cookie)
            .map_err(|_| ApiError::InternalServerError("".to_string()))?,
    );

    let response = AuthResponse {
        url: full_url.clone(),
    };
    let json_body = serde_json::to_string(&response)
        .map_err(|_| ApiError::InternalServerError("Failed to serialize response".to_string()))?;

    let axum_response = axum::response::Response::builder()
        .status(StatusCode::FOUND)
        .header(http::header::SET_COOKIE, cookie_value)
        .header(SET_COOKIE, session_cookie)
        .header(LOCATION, full_url)
        .header(ORIGIN, state.args.webapp_url.clone())
        .body(axum::body::Body::from(json_body))
        .map_err(|_| ApiError::InternalServerError("Failed to build response".to_string()))?;

    Ok(axum_response)
}

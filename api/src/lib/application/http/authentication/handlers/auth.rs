use axum::http::header::{LOCATION, ORIGIN};
use axum::{
    extract::{Query, State},
    http::{
        self, HeaderMap, HeaderValue, StatusCode,
        header::{CONTENT_TYPE, SET_COOKIE},
    },
    response::IntoResponse,
};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use validator::Validate;

use crate::domain::authentication::entities::dto::CreateAuthSessionDto;
use crate::{
    application::http::server::{api_entities::api_error::ApiError, app_state::AppState},
    domain::{
        authentication::ports::auth_session::AuthSessionService,
        client::ports::{client_service::ClientService, redirect_uri_service::RedirectUriService},
        realm::ports::realm_service::RealmService,
    },
};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
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
#[typeshare]
pub struct AuthResponse {
    pub url: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/protocol/openid-connect/auth")]
pub struct AuthRoute {
    pub realm_name: String,
}

#[utoipa::path(get, path = "/protocol/openid-connect/auth", tag = "auth")]
pub async fn auth(
    AuthRoute { realm_name }: AuthRoute,
    State(state): State<AppState>,
    Query(params): Query<AuthRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(|_| ApiError::InternalServerError("".to_string()))?;

    let client = state
        .client_service
        .get_by_client_id(params.client_id.clone(), realm.id)
        .await
        .map_err(|_| ApiError::InternalServerError("".to_string()))?;

    let redirect_uri = params.redirect_uri.clone();

    let client_redirect_uris = state
        .redirect_uri_service
        .get_enabled_by_client_id(client.id)
        .await
        .map_err(|_| ApiError::InternalServerError("".to_string()))?;

    if !client_redirect_uris.iter().any(|uri| {
        // Check for exact match first
        if uri.value == redirect_uri {
            return true;
        }

        // If not an exact match, try to interpret as regex pattern
        if let Ok(regex) = regex::Regex::new(&uri.value) {
            return regex.is_match(&redirect_uri);
        }

        false
    }) {
        return Err(ApiError::Unauthorized("Invalid redirect_uri".to_string()));
    }

    if !client.enabled {
        return Err(ApiError::Unauthorized("Client is disabled".to_string()));
    }

    let dto = CreateAuthSessionDto::new(realm.id, client.id, redirect_uri).with_oauth_params(
        params.response_type,
        params.scope.unwrap_or_default(),
        params.state.clone(),
        None,
    );

    let session = state
        .auth_session_service
        .create_session(dto)
        .await
        .map_err(
            |e: crate::domain::authentication::entities::auth_session::AuthSessionError| {
                ApiError::InternalServerError(e.to_string())
            },
        )?;

    let login_url = format!(
        "?client_id={}&redirect_uri={}&state={}",
        client.client_id,
        params.redirect_uri,
        params.state.unwrap_or_default()
    );

    let full_url = format!(
        "{}/realms/{}/authentication/login{}",
        state.env.portal_url.clone(),
        realm.name,
        login_url.clone()
    );

    let cookie_value = format!(
        "session_code={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=3600",
        session.id
    );

    let session_cookie = format!(
        "FERRISKEY_SESSION={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=3600",
        session.id
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
        .header(ORIGIN, state.env.portal_url.clone())
        .body(axum::body::Body::from(json_body))
        .map_err(|_| ApiError::InternalServerError("Failed to build response".to_string()))?;

    Ok(axum_response)
}

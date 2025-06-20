use axum::extract::{Query, State};
use axum_cookie::CookieManager;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::domain::authentication::entities::jwt_token::JwtToken;
use crate::domain::authentication::ports::auth_session::AuthSessionService;
use crate::domain::authentication::ports::authentication::AuthenticationService;
use crate::domain::realm::ports::realm_service::RealmService;
use crate::domain::user::ports::user_service::UserService;

#[derive(Serialize, Deserialize)]
#[typeshare]
pub struct AuthenticateQueryParams {
    client_id: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[typeshare]
pub struct AuthenticateResponse {
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[typeshare]
pub struct AuthenticateRequest {
    #[validate(length(min = 1, message = "username is required"))]
    #[serde(default)]
    pub username: String,

    #[validate(length(min = 1, message = "password is required"))]
    #[serde(default)]
    pub password: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/login-actions/authenticate")]
pub struct TokenRoute {
    realm_name: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/authenticate",
    tag = "auth",
    request_body = AuthenticateRequest,
    responses(
        (status = 200, body = JwtToken)
    )
)]
pub async fn authenticate(
    TokenRoute { realm_name }: TokenRoute,
    State(state): State<AppState>,
    Query(query): Query<AuthenticateQueryParams>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<AuthenticateRequest>,
) -> Result<Response<AuthenticateResponse>, ApiError> {
    let session_code = cookie.get("FERRISKEY_SESSION").unwrap();
    let session_code = session_code.value().to_string();

    let session_code = Uuid::parse_str(&session_code).unwrap();

    let auth_session = state
        .auth_session_service
        .get_by_session_code(session_code)
        .await
        .map_err(|_| ApiError::Unauthorized("invalid session code".to_string()))?;

    let code = state
        .authentication_service
        .using_session_code(
            realm_name.clone(),
            query.client_id,
            auth_session.id,
            payload.username.clone(),
            payload.password,
        )
        .await
        .map_err(|_| ApiError::Unauthorized("invalid credentials".to_string()))?;

    let realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(|_| ApiError::Unauthorized("invalid realm".to_string()))?;

    let user = state
        .user_service
        .get_by_username(payload.username, realm.id)
        .await
        .map_err(|_| ApiError::Unauthorized("invalid credentials".to_string()))?;

    let code = if auth_session.code.is_none() {
        let t = state
            .auth_session_service
            .update_code(session_code, code.clone(), user.id)
            .await
            .map_err(|_| ApiError::Unauthorized("invalid credentials".to_string()))?;

        t.code.clone().unwrap_or_default()
    } else {
        auth_session.code.clone().unwrap_or_default()
    };

    let current_state = auth_session
        .state
        .ok_or(ApiError::Unauthorized("invalid credentials".to_string()))?;

    let login_url = format!(
        "{}?code={}&state={}",
        auth_session.redirect_uri, code, current_state
    );

    let response = AuthenticateResponse { url: login_url };

    Ok(Response::OK(response))
}

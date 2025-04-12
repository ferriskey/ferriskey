use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
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
use crate::domain::authentication::entities::error::AuthenticationError;
use crate::domain::authentication::entities::jwt_token::JwtToken;
use crate::domain::authentication::ports::auth_session::AuthSessionService;
use crate::domain::authentication::ports::authentication::AuthenticationService;

#[derive(Serialize, Deserialize)]
#[typeshare]
pub struct AuthenticateQueryParams {
    client_id: String,
    // #[typeshare(serialized_as = "string")]
    // session_code: Uuid,
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
    // get session_code from cookies
    let session_code = cookie.get("session_code").unwrap();
    let session_code = session_code.value().to_string();
    println!("session_code: {}", session_code);
    let session_code = Uuid::parse_str(&session_code).unwrap();
    let auth_session = state
        .auth_session_service
        .get_by_session_code(session_code)
        .await
        .map_err(|_| AuthenticationError::NotFound)?;

    let code = state
        .authentication_service
        .using_session_code(
            realm_name,
            query.client_id,
            auth_session.id,
            payload.username,
            payload.password,
        )
        .await?;

    let current_state = auth_session
        .state
        .ok_or(AuthenticationError::InvalidState)?;

    let login_url = format!(
        "{}?code={}&state={}",
        auth_session.redirect_uri, code, current_state
    );

    let response = AuthenticateResponse { url: login_url };

    Ok(Response::OK(response))
}

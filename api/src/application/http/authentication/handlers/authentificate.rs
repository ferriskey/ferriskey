use crate::application::decoded_token::OptionalToken;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::app_state::AppState;
use crate::application::url::FullUrl;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header::SET_COOKIE};
use axum::response::IntoResponse;
use axum_cookie::CookieManager;
use axum_extra::extract::cookie::{Cookie, SameSite};

use ferriskey_core::domain::authentication::entities::{
    AuthenticateInput, AuthenticateOutput, AuthenticationStepStatus, ExchangeTokenInput, GrantType,
};
use ferriskey_core::domain::authentication::ports::AuthService;
use ferriskey_core::domain::user::entities::RequiredAction;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct AuthenticateQueryParams {
    client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
pub enum AuthenticationStatus {
    Success,
    RequiresActions,
    RequiresOtpChallenge,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
pub struct AuthenticateResponse {
    pub status: AuthenticationStatus,
    pub url: Option<String>,
    pub required_actions: Option<Vec<RequiredAction>>,
    pub token: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AuthenticateRequest {
    #[validate(length(min = 1, message = "username is required"))]
    #[serde(default)]
    pub username: Option<String>,

    #[validate(length(min = 1, message = "password is required"))]
    #[serde(default)]
    pub password: Option<String>,
}

impl From<AuthenticateOutput> for AuthenticateResponse {
    fn from(result: AuthenticateOutput) -> Self {
        match result.status {
            AuthenticationStepStatus::Success => AuthenticateResponse {
                status: AuthenticationStatus::Success,
                url: result.redirect_url,
                required_actions: None,
                token: None,
                message: Some("Authentication successful".to_string()),
            },
            AuthenticationStepStatus::RequiresActions => AuthenticateResponse {
                status: AuthenticationStatus::RequiresActions,
                url: None,
                required_actions: if result.required_actions.is_empty() {
                    None
                } else {
                    Some(result.required_actions)
                },
                token: result.temporary_token,
                message: Some("Additional actions required before login".to_string()),
            },
            AuthenticationStepStatus::RequiresOtpChallenge => AuthenticateResponse {
                status: AuthenticationStatus::RequiresOtpChallenge,
                url: None,
                required_actions: None,
                token: result.temporary_token,
                message: Some("OTP verification required".to_string()),
            },
            AuthenticationStepStatus::Failed => AuthenticateResponse {
                status: AuthenticationStatus::Failed,
                url: None,
                required_actions: None,
                token: None,
                message: Some("Authentication failed".to_string()),
            },
        }
    }
}

#[utoipa::path(
    post,
    path = "/login-actions/authenticate",
    tag = "auth",
    summary = "Authenticate a user in a realm",
    request_body = AuthenticateRequest,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = AuthenticateResponse)
    )
)]
pub async fn authenticate(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    OptionalToken(optional_token): OptionalToken,
    Query(query): Query<AuthenticateQueryParams>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<AuthenticateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let session_code = match cookie.get("FERRISKEY_SESSION") {
        Some(cookie) => cookie,
        None => return Err(ApiError::Unauthorized("Missing session cookie".to_string())),
    };
    let session_code = session_code.value().to_string();

    let session_code = Uuid::parse_str(&session_code)
        .map_err(|_| ApiError::BadRequest("Invalid session code in cookie".to_string()))?;

    let authenticate_params = if let Some(token) = optional_token {
        AuthenticateInput::with_existing_token(
            realm_name.clone(),
            query.client_id.clone(),
            session_code,
            base_url.clone(),
            token.token,
        )
    } else {
        let username = payload
            .username
            .clone()
            .ok_or_else(|| ApiError::BadRequest("username is required".to_string()))?;
        let password = payload
            .password
            .clone()
            .ok_or_else(|| ApiError::BadRequest("password is required".to_string()))?;

        AuthenticateInput::with_user_credentials(
            realm_name.clone(),
            query.client_id.clone(),
            session_code,
            base_url.clone(),
            username,
            password,
        )
    };
    let result = state.service.authenticate(authenticate_params).await?;

    let mut identity_cookie_value: Option<HeaderValue> = None;
    let is_secure = base_url.starts_with("https://");
    if result.status == AuthenticationStepStatus::Success
        && let Some(code) = result.authorization_code.clone()
        && let Ok(jwt_token) = state
            .service
            .exchange_token(ExchangeTokenInput {
                realm_name: realm_name.clone(),
                client_id: query.client_id.clone(),
                client_secret: None,
                code: Some(code),
                username: None,
                password: None,
                refresh_token: None,
                base_url: base_url.clone(),
                grant_type: GrantType::Code,
                scope: None,
            })
            .await
    {
        let mut identity_cookie =
            Cookie::build(("FERRISKEY_IDENTITY", jwt_token.access_token().to_string()))
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax);

        if is_secure {
            identity_cookie = identity_cookie.secure(true);
        }

        let cookie_value = HeaderValue::from_str(&identity_cookie.to_string())
            .map_err(|_| ApiError::InternalServerError("Invalid cookie header".to_string()))?;
        identity_cookie_value = Some(cookie_value);
    }

    let response: AuthenticateResponse = result.into();
    if let Some(cookie_value) = identity_cookie_value {
        return Ok((
            StatusCode::OK,
            [(SET_COOKIE, cookie_value)],
            axum::Json(response),
        )
            .into_response());
    }

    Ok((StatusCode::OK, axum::Json(response)).into_response())
}

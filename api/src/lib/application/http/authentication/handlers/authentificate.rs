use axum::extract::{Query, State};
use axum_cookie::CookieManager;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::application::decoded_token::OptionalToken;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::application::url::FullUrl;

use crate::domain::authentication::use_cases::entities::{
    AuthenticateCommand, AuthenticateResult, AuthenticationStepStatus,
};

use crate::domain::user::entities::required_action::RequiredAction;

#[derive(Serialize, Deserialize)]
#[typeshare]
pub struct AuthenticateQueryParams {
    client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
#[typeshare]
pub enum AuthenticationStatus {
    Success,
    RequiresActions,
    RequiresOtpChallenge,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
#[typeshare]
pub struct AuthenticateResponse {
    pub status: AuthenticationStatus,
    pub url: Option<String>,
    pub required_actions: Option<Vec<RequiredAction>>,
    pub token: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[typeshare]
pub struct AuthenticateRequest {
    #[validate(length(min = 1, message = "username is required"))]
    #[serde(default)]
    pub username: Option<String>,

    #[validate(length(min = 1, message = "password is required"))]
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/login-actions/authenticate")]
pub struct TokenRoute {
    realm_name: String,
}

impl From<AuthenticateResult> for AuthenticateResponse {
    fn from(result: AuthenticateResult) -> Self {
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
    request_body = AuthenticateRequest,
    responses(
        (status = 200, body = AuthenticateResponse)
    )
)]
pub async fn authenticate(
    TokenRoute { realm_name }: TokenRoute,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    OptionalToken(optional_token): OptionalToken,
    Query(query): Query<AuthenticateQueryParams>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<AuthenticateRequest>,
) -> Result<Response<AuthenticateResponse>, ApiError> {
    let session_code = cookie.get("FERRISKEY_SESSION").unwrap();
    let session_code = session_code.value().to_string();

    let session_code = Uuid::parse_str(&session_code).unwrap();

    let command = if let Some(token) = optional_token {
        AuthenticateCommand::with_existing_token(
            realm_name,
            query.client_id,
            session_code,
            base_url,
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

        AuthenticateCommand::with_user_credentials(
            realm_name.clone(),
            query.client_id.clone(),
            session_code,
            base_url.clone(),
            username,
            password,
        )
    };

    let result = state
        .authenticate_use_case
        .execute(command)
        .await
        .map_err(|_| ApiError::Unauthorized("invalid credentials".to_string()))?;

    let response: AuthenticateResponse = result.into();

    Ok(Response::OK(response))
}

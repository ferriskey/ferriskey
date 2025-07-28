use axum::extract::{Query, State};
use axum_cookie::CookieManager;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::application::decoded_token::OptionalToken;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::application::url::FullUrl;
use crate::domain::authentication::ports::{
    auth_session::AuthSessionService, authentication::AuthenticationService,
};
use crate::domain::jwt::ports::jwt_service::JwtService;
use crate::domain::realm::ports::realm_service::RealmService;
use crate::domain::user::entities::required_action::RequiredAction;
use crate::domain::utils::generate_random_string;

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

    info!("Authenticating user with session code: {}", session_code);

    let auth_session = state
        .auth_session_service
        .get_by_session_code(session_code)
        .await
        .map_err(|_| ApiError::Unauthorized("invalid session code".to_string()))?;

    let realm = state
        .realm_service
        .get_by_name(realm_name.clone())
        .await
        .map_err(|_| ApiError::Unauthorized("invalid realm name".to_string()))?;

    if let Some(result_token) = optional_token {
        info!("optional_token present");
        state
            .jwt_service
            .verify_token(result_token.token, realm.id)
            .await
            .map_err(|e| {
                error!("invalid token: {}", e);
                ApiError::Unauthorized("invalid token".to_string())
            })?;

        let auth_session = state
            .auth_session_service
            .get_by_session_code(session_code)
            .await
            .map_err(|_| ApiError::Unauthorized("invalid session code".to_string()))?;

        let code = generate_random_string();
        let user_id = Uuid::parse_str(&result_token.decoded_token.sub)
            .map_err(|_| ApiError::Unauthorized("invalid sub id".to_string()))?;
        state
            .auth_session_service
            .update_code(session_code, code.clone(), user_id)
            .await
            .map_err(|_| ApiError::Unauthorized("invalid session code".to_string()))?;

        let current_state = auth_session
            .state
            .ok_or(ApiError::Unauthorized("Invlaid session state".to_string()))?;
        let login_url = format!(
            "{}?code={}&state={}",
            auth_session.redirect_uri, code, current_state
        );

        let response = AuthenticateResponse {
            status: AuthenticationStatus::Success,
            url: Some(login_url),
            required_actions: None,
            message: None,
            token: None,
        };

        return Ok(Response::OK(response));
    }

    let username = payload
        .username
        .ok_or(ApiError::Unauthorized("username is required".to_string()))?;

    let password = payload
        .password
        .ok_or(ApiError::Unauthorized("password is required".to_string()))?;

    let auth_result = state
        .authentication_service
        .using_session_code(
            realm_name.clone(),
            query.client_id,
            auth_session.id,
            username,
            password,
            base_url,
        )
        .await
        .map_err(|_| ApiError::Unauthorized("invalid credentials".to_string()))?;

    if !auth_result.required_actions.is_empty() {
        let response = AuthenticateResponse {
            status: AuthenticationStatus::RequiresActions,
            url: None,
            required_actions: Some(auth_result.required_actions),
            message: Some("Additional actions required before login".to_string()),
            token: auth_result.token,
        };

        return Ok(Response::OK(response));
    }

    let has_otp_credentials = auth_result.credentials.iter().any(|cred| cred == "otp");

    if has_otp_credentials
        && !auth_result
            .required_actions
            .contains(&RequiredAction::ConfigureOtp)
    {
        let response = AuthenticateResponse {
            status: AuthenticationStatus::RequiresOtpChallenge,
            url: None,
            required_actions: None,
            message: Some("OTP verification required".to_string()),
            token: auth_result.token,
        };

        return Ok(Response::OK(response));
    }

    let code = auth_result.code.ok_or(ApiError::Unauthorized(
        "No authorization code generated".to_string(),
    ))?;

    if auth_session.code.is_none() {
        state
            .auth_session_service
            .update_code(session_code, code.clone(), auth_result.user_id)
            .await
            .map_err(|_| ApiError::Unauthorized("Failed to update session".to_string()))?;
    }

    let current_state = auth_session
        .state
        .ok_or(ApiError::Unauthorized("Invalid session state".to_string()))?;

    let login_url = format!(
        "{}?code={}&state={}",
        auth_session.redirect_uri, code, current_state
    );

    let response = AuthenticateResponse {
        status: AuthenticationStatus::Success,
        url: Some(login_url),
        required_actions: None,
        message: None,
        token: auth_result.token,
    };

    Ok(Response::OK(response))
}

use super::auth::root_scoped_base_url;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_cookie::CookieManager;
use ferriskey_api_core::api_entities::api_error::{ApiError, ValidateJson};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::decoded_token::OptionalToken;
use ferriskey_api_core::url::FullUrl;

use ferriskey_core::domain::authentication::entities::AuthenticateInput;
use ferriskey_core::domain::authentication::entities::AuthenticationStepStatus;
use ferriskey_core::domain::authentication::ports::AuthService;
use ferriskey_core::domain::email_verification::ports::EmailVerificationService;
use ferriskey_core::domain::user::entities::RequiredAction;
use serde::{Deserialize, Serialize};
use tracing::warn;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

pub use ferriskey_api_core::authentication::{AuthenticateResponse, AuthenticationStatus};

#[derive(Serialize, Deserialize)]
pub struct AuthenticateQueryParams {
    client_id: String,
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
        (status = 200, description = "Authenticated", body = AuthenticateResponse),
        (status = 400, description = "Invalid input", body = ApiError),
        (status = 401, description = "Missing session cookie", body = ApiError),
        (status = 401, description = "Invalid realm", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
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
        None => return Err(ApiError::Unauthorized("Missing session cookie".into())),
    };
    let session_code = session_code.value().to_string();

    let session_code = Uuid::parse_str(&session_code)
        .map_err(|_| ApiError::BadRequest("Invalid session code in cookie".into()))?;

    let base_url = root_scoped_base_url(&base_url, &state.args.server.root_path);

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
            .ok_or_else(|| ApiError::BadRequest("username is required".into()))?;
        let password = payload
            .password
            .clone()
            .ok_or_else(|| ApiError::BadRequest("password is required".into()))?;

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

    // If user has VerifyEmail required action, automatically send verification email
    if result.status == AuthenticationStepStatus::RequiresActions
        && result
            .required_actions
            .contains(&RequiredAction::VerifyEmail)
    {
        let verification_base_url = state.args.webapp_url.trim_end_matches('/').to_string();

        if let Err(e) = state
            .service
            .email_verification_service
            .send_verification_email(result.user_id, realm_name.clone(), verification_base_url)
            .await
        {
            // Log the error but don't fail the authentication - user can still use resend button
            warn!(
                user_id = %result.user_id,
                realm = %realm_name,
                error = %e,
                "Failed to send verification email during authentication"
            );
        }
    }

    let response: AuthenticateResponse = result.into();
    Ok((StatusCode::OK, axum::Json(response)).into_response())
}

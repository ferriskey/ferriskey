use axum::extract::{Path, Query, State};
use axum_extra::extract::CookieJar;
use ferriskey_core::domain::{
    authentication::entities::AuthenticateInput,
    authentication::ports::AuthService,
    trident::ports::{MagicLinkInput, TridentService},
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::application::{
    http::server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response as ApiResponse,
        },
        app_state::AppState,
    },
    url::FullUrl,
};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct MagicLinkRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MagicLinkResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyMagicLinkResponse {
    pub url: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyMagicLinkQuery {
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/realms/{realm_name}/login-actions/send_magic_link",
    tag = "auth",
    summary = "Send magic link",
    description = "Generates and sends a magic link for authentication",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = MagicLinkRequest,
    responses(
        (status = 200, body = MagicLinkResponse, description = "Magic link sent successfully"),
        (status = 400, description = "Bad Request - Invalid email"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn send_magic_link(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<MagicLinkRequest>,
) -> Result<ApiResponse<MagicLinkResponse>, ApiError> {
    debug!(
        "Sending magic link for email: {} in realm: {}",
        payload.email, realm_name
    );

    state
        .service
        .generate_magic_link(MagicLinkInput {
            realm_name,
            email: payload.email,
        })
        .await?;

    Ok(ApiResponse::OK(MagicLinkResponse {
        message: "Magic link sent successfully".to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/realms/{realm_name}/login-actions/verify-magic-link",
    tag = "auth",
    summary = "Verify magic link",
    description = "Verifies a magic link token and authenticates the user using the unified authentication flow",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("token" = String, Query, description = "Magic link token"),
    ),
    responses(
        (status = 200, body = VerifyMagicLinkResponse, description = "Magic link verified successfully"),
        (status = 400, description = "Bad Request - Invalid token"),
        (status = 401, description = "Unauthorized - Missing session cookie"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn verify_magic_link(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<VerifyMagicLinkQuery>,
    jar: CookieJar,
    FullUrl(_, base_url): FullUrl,
) -> Result<ApiResponse<VerifyMagicLinkResponse>, ApiError> {
    // Extract session code from cookie
    let session_code = jar
        .get("FERRISKEY_SESSION")
        .ok_or_else(|| ApiError::Unauthorized("Missing session cookie".to_string()))?
        .value()
        .parse::<Uuid>()
        .map_err(|_| ApiError::BadRequest("Invalid session code".to_string()))?;

    debug!(
        "Verifying magic link token: {} for session: {} in realm: {}",
        query.token, session_code, realm_name
    );

    let authenticate_input = AuthenticateInput::with_magic_token(
        realm_name,
        "security-admin-console".to_string(), // TODO Get from query params or session
        session_code,
        base_url,
        query.token,
    );

    let result = state.service.authenticate(authenticate_input).await?;

    let response = VerifyMagicLinkResponse {
        url: result.redirect_url,
        message: match result.status {
            ferriskey_core::domain::authentication::entities::AuthenticationStepStatus::Success =>
                Some("Magic link authentication successful".to_string()),
            ferriskey_core::domain::authentication::entities::AuthenticationStepStatus::RequiresActions =>
                Some("Additional actions required before login".to_string()),
            ferriskey_core::domain::authentication::entities::AuthenticationStepStatus::RequiresOtpChallenge =>
                Some("OTP verification required".to_string()),
            ferriskey_core::domain::authentication::entities::AuthenticationStepStatus::Failed =>
                Some("Magic link authentication failed".to_string()),
        },
    };

    Ok(ApiResponse::OK(response))
}

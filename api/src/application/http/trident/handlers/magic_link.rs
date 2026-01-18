use crate::application::http::server::api_entities::response::Response;
use axum::extract::{Path, Query, State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::{
    authentication::{entities::AuthenticateInput, ports::AuthService},
    trident::ports::{MagicLinkInput, TridentService, VerifyMagicLinkInput},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::application::{
    http::{
        authentication::handlers::authentificate::AuthenticateResponse,
        server::{
            api_entities::api_error::{ApiError, ValidateJson},
            app_state::AppState,
        },
    },
    url::FullUrl,
};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SendMagicLinkRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SendMagicLinkResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyMagicLinkQuery {
    client_id: String,
    token_id: String,
    magic_token: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/send-magic-link",
    tag = "auth",
    summary = "Send magic link for passwordless authentication",
    description = "Sends a magic link to the user's email for passwordless authentication. The link contains a unique token that can be used to verify the user's identity.",
    params(
        ("realm_name" = String, Path, description = "The realm name"),
    ),
    request_body = SendMagicLinkRequest,
    responses(
        (status = 200, body = SendMagicLinkResponse, description = "Magic link sent successfully"),
        (status = 400, description = "Bad Request - Invalid email format"),
        (status = 404, description = "Not Found - User not found in realm"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn send_magic_link(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<SendMagicLinkRequest>,
) -> Result<Response<SendMagicLinkResponse>, ApiError> {
    debug!(
        "Generating magic link for email: {} in realm: {}",
        payload.email, realm_name
    );

    state
        .service
        .generate_magic_link(MagicLinkInput {
            realm_name,
            email: payload.email.clone(),
        })
        .await?;

    debug!("Magic link sent successfully to email: {}", payload.email);

    Ok(Response::OK(SendMagicLinkResponse {
        message: "Magic link sent successfully. Check your email.".to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/login-actions/verify-magic-link",
    tag = "auth",
    summary = "Verify magic link and complete authentication",
    description = "Verifies the magic link token and completes the authentication flow. Returns authentication status and optional redirect URL with authorization code.",
    params(
        ("realm_name" = String, Path, description = "The realm name"),
        ("token_id" = String, Query, description = "The unique token identifier from the magic link"),
        ("magic_token" = String, Query, description = "The secret verification token from the magic link"),
        ("client_id" = String, Query, description = "The OAuth 2.0 client identifier"),
    ),
    responses(
        (status = 200, body = AuthenticateResponse, description = "Magic link verified successfully"),
        (status = 400, description = "Bad Request - Invalid session code or parameters"),
        (status = 401, description = "Unauthorized - Missing or invalid session cookie"),
        (status = 404, description = "Not Found - Magic link not found or expired"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn verify_magic_link(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    Query(query): Query<VerifyMagicLinkQuery>,
    cookie: CookieManager,
) -> Result<Response<AuthenticateResponse>, ApiError> {
    let session_code = match cookie.get("FERRISKEY_SESSION") {
        Some(cookie) => cookie,
        None => {
            warn!("Magic link verification attempted without session cookie");
            return Err(ApiError::Unauthorized("Missing session cookie".to_string()));
        }
    };

    let session_code = session_code.value().to_string();

    let session_code = Uuid::parse_str(&session_code).map_err(|_| {
        warn!("Failed to parse session code from cookie");
        ApiError::BadRequest("Invalid session code in cookie".to_string())
    })?;

    debug!(
        "Verifying magic link for token_id: {} in realm: {}",
        query.token_id, realm_name
    );

    let login_url = state
        .service
        .verify_magic_link(VerifyMagicLinkInput {
            magic_token_id: query.token_id,
            magic_token: query.magic_token,
            session_code: session_code.to_string(),
        })
        .await?;

    let auth_input = AuthenticateInput::with_magic_token(
        realm_name,
        query.client_id,
        session_code,
        base_url,
        login_url,
    );

    let result = state.service.authenticate(auth_input).await?;
    Ok(Response::OK(result.into()))
}

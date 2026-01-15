use crate::application::http::server::api_entities::response::Response;
use axum::extract::{Path, Query, State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::{
    authentication::{entities::AuthenticateInput, ports::AuthService},
    trident::ports::{MagicLinkInput, TridentService, VerifyMagicLinkInput},
};
use serde::{Deserialize, Serialize};
use tracing::debug;
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
pub struct MagicLinkRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyMagicLinkQuery {
    client_id: String,
    token_id: String,
    magic_token: String,
}

#[utoipa::path(
    post,
    path = "/realms/{realm_name}/login-actions/send_magic_link",
    tag = "auth",
    summary = "Log magic link",
    description = "Generates and logs a magic link for authentication",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = MagicLinkRequest,
    responses(
        (status = 200),
        (status = 400),
        (status = 500)
    )
)]
pub async fn send_magic_link(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<MagicLinkRequest>,
) -> Result<Response<()>, ApiError> {
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

    Ok(Response::OK(()))
}

#[utoipa::path(
    get,
    path = "/realms/{realm_name}/login-actions/verify-magic-link",
    tag = "auth",
    summary = "Verify magic link and authenticate user",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("token_id" = String, Query, description = "Magic link token identifier"),
        ("magic_token" = String, Query, description = "Magic link secret token"),
    ),
    responses(
        (status = 200, body = AuthenticateResponse, description = "Magic link verified successfully"),
        (status = 400),
        (status = 401),
        (status = 500)
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
        None => return Err(ApiError::Unauthorized("Missing session cookie".to_string())),
    };

    let session_code = session_code.value().to_string();

    let session_code = Uuid::parse_str(&session_code)
        .map_err(|_| ApiError::BadRequest("Invalid session code in cookie".to_string()))?;

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

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header::LOCATION},
    response::{IntoResponse, Response},
};
use axum_cookie::CookieManager;
use base64::{Engine, engine::general_purpose};
use ferriskey_core::domain::authentication::entities::AuthorizeRequestInput;
use ferriskey_core::domain::authentication::ports::AuthService;
use ferriskey_core::domain::jwt::entities::JwtClaim;
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};
use utoipa::ToSchema;

use crate::application::http::server::api_entities::api_error::{ApiError, ApiErrorResponse};
use crate::application::http::server::app_state::AppState;

const IDENTITY_COOKIE: &str = "FERRISKEY_IDENTITY";

#[derive(Debug, Deserialize)]
pub struct DevicePageQuery {
    /// Pre-fills the verification form (from `verification_uri_complete`).
    pub user_code: Option<String>,
}

/// Approve or deny choice submitted from the verification page.
#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DeviceVerifyAction {
    Approve,
    Deny,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeviceVerifyRequest {
    /// The end-user code shown on the device (e.g. `WDJB-MJHT`).
    pub user_code: String,
    pub action: DeviceVerifyAction,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceVerifyResponse {
    /// Resulting session status: `approved` or `denied`.
    pub status: String,
}

#[utoipa::path(
    get,
    path = "/device",
    tag = "auth",
    summary = "Device verification page",
    description = "Entry point the user visits (the `verification_uri`) to approve a device. Redirects to the FerrisKey web app, pre-filling the user code when supplied via `?user_code=`.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_code" = Option<String>, Query, description = "User code to pre-fill"),
    ),
    responses(
        (status = 302, description = "Redirect to the web app verification page"),
    )
)]
#[instrument(skip(state), fields(realm_name = %realm_name))]
pub async fn device_verification_page(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<DevicePageQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let mut url = format!(
        "{}/realms/{}/authentication/device",
        state.args.webapp_url.trim_end_matches('/'),
        realm_name
    );

    if let Some(user_code) = query.user_code.as_deref().filter(|c| !c.trim().is_empty()) {
        url = format!("{url}?user_code={}", urlencoding::encode(user_code));
    }

    Ok((StatusCode::FOUND, [(LOCATION, url)]))
}

#[utoipa::path(
    post,
    path = "/device/verify",
    tag = "auth",
    summary = "Approve or deny a device authorization",
    description = "Called from the verification page once the user is authenticated. Requires the `FERRISKEY_IDENTITY` cookie; when absent, responds 401 with a `redirect_uri` hint pointing back to the verification page so the front can route to login first.",
    request_body = DeviceVerifyRequest,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Device authorization updated", body = DeviceVerifyResponse),
        (status = 400, description = "Unknown or expired user code", body = ApiErrorResponse),
        (status = 401, description = "Not logged in — redirect hint provided"),
        (status = 403, description = "Service accounts cannot approve devices", body = ApiErrorResponse),
    )
)]
#[instrument(skip(state, cookie, payload), fields(realm_name = %realm_name, action = ?payload.action))]
pub async fn device_verify(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    cookie: CookieManager,
    Json(payload): Json<DeviceVerifyRequest>,
) -> Result<Response, ApiError> {
    // Require a non-empty identity cookie; otherwise hint the front to log in
    // and come back to this verification page.
    let token = cookie
        .get(IDENTITY_COOKIE)
        .map(|c| c.value().to_string())
        .filter(|value| !value.trim().is_empty());

    let token = match token {
        Some(token) => token,
        None => {
            let redirect_uri = format!(
                "/realms/{}/authentication/device?user_code={}",
                realm_name,
                urlencoding::encode(&payload.user_code)
            );
            return Ok((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "login_required",
                    "error_description": "Authentication required to approve this device.",
                    "redirect_uri": redirect_uri,
                })),
            )
                .into_response());
        }
    };

    // Validate the identity token and resolve the acting user.
    let claims = decode_jwt_claims(&token)
        .ok_or_else(|| ApiError::Unauthorized("Invalid identity token".into()))?;

    let output = state
        .service
        .authorize_request(AuthorizeRequestInput { claims, token })
        .await
        .map_err(|error| {
            warn!(error = ?error, "Device verify: identity token rejected");
            ApiError::Unauthorized("Invalid identity token".into())
        })?;

    let user = output
        .identity
        .as_user()
        .ok_or_else(|| ApiError::Forbidden("Service accounts cannot approve devices".into()))?;
    let user_id = user.id;

    let status = match payload.action {
        DeviceVerifyAction::Approve => {
            state
                .service
                .verify_device_user_code(payload.user_code, user_id)
                .await?;
            "approved"
        }
        DeviceVerifyAction::Deny => {
            state
                .service
                .deny_device_user_code(payload.user_code, user_id)
                .await?;
            "denied"
        }
    };

    Ok((
        StatusCode::OK,
        Json(DeviceVerifyResponse {
            status: status.to_string(),
        }),
    )
        .into_response())
}

/// Decode (without verifying) the claims segment of a JWT. Signature
/// verification happens in `authorize_request`.
fn decode_jwt_claims(token: &str) -> Option<JwtClaim> {
    let payload = token.split('.').nth(1)?;
    let decoded = general_purpose::URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice(&decoded).ok()
}

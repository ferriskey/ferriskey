use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header::LOCATION},
    response::{IntoResponse, Response},
};
use axum_cookie::CookieManager;
use ferriskey_core::domain::authentication::device_flow::DeviceVerificationPreview;
use ferriskey_core::domain::authentication::ports::AuthService;
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};
use utoipa::ToSchema;

use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::sso_cookie::SSO_SESSION_COOKIE;
use ferriskey_core::domain::user::entities::User;

/// The signed-in user behind the browser's `FERRISKEY_SSO` cookie, if any.
async fn signed_in_user(
    state: &AppState,
    cookie: &CookieManager,
    realm_name: &str,
) -> Option<User> {
    let sso_cookie = cookie
        .get(SSO_SESSION_COOKIE)
        .map(|c| c.value().trim().to_string())
        .filter(|value| !value.is_empty())?;

    state
        .service
        .resolve_sso_user(realm_name.to_string(), sso_cookie)
        .await
        .inspect_err(|error| warn!(error = ?error, "Device flow: SSO session refused"))
        .ok()
}

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

#[derive(Debug, Deserialize)]
pub struct DevicePreviewQuery {
    pub user_code: String,
}

#[utoipa::path(
    get,
    path = "/device/preview",
    tag = "auth",
    summary = "Device consent preview",
    description = "Returns the client and the scopes a pending device session is asking for, so the verification page can show what is being approved (RFC 8628 §5.3). Requires the `FERRISKEY_SSO` session cookie and refuses codes belonging to another realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_code" = String, Query, description = "The end-user code shown on the device"),
    ),
    responses(
        (status = 200, description = "Pending session details", body = DeviceVerificationPreview),
        (status = 401, description = "Authentication required", body = ApiErrorResponse),
        (status = 403, description = "The session belongs to another realm", body = ApiErrorResponse),
    )
)]
#[instrument(skip(state, cookie), fields(realm_name = %realm_name))]
pub async fn device_preview(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    cookie: CookieManager,
    Query(query): Query<DevicePreviewQuery>,
) -> Result<Response, ApiError> {
    let user = signed_in_user(&state, &cookie, &realm_name)
        .await
        .ok_or_else(|| ApiError::Unauthorized("Authentication required".into()))?;

    let preview = state
        .service
        .describe_device_user_code(realm_name, query.user_code, user.realm_id)
        .await?;

    Ok((StatusCode::OK, Json(preview)).into_response())
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
    description = "Called from the verification page once the user is authenticated. Requires the `FERRISKEY_SSO` session cookie; when absent or no longer valid, responds 401 with a `redirect_uri` hint pointing back to the verification page so the front can route to login first.",
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
    // Without a live SSO session, hint the front to log in and come back to
    // this verification page.
    let Some(user) = signed_in_user(&state, &cookie, &realm_name).await else {
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
    };
    let user_id = user.id;

    let status = match payload.action {
        DeviceVerifyAction::Approve => {
            state
                .service
                .verify_device_user_code(
                    realm_name.clone(),
                    payload.user_code,
                    user_id,
                    user.realm_id,
                )
                .await?;
            "approved"
        }
        DeviceVerifyAction::Deny => {
            state
                .service
                .deny_device_user_code(
                    realm_name.clone(),
                    payload.user_code,
                    user_id,
                    user.realm_id,
                )
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

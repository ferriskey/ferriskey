use axum::extract::{Path, State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::trident::ports::{RequestPasswordResetInput, TridentService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct ForgotPasswordResponse;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(email)]
    pub email: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/forgot-password",
    tag = "auth",
    summary = "Request a password reset",
    description = "Sends a password reset email to the user if the email exists in the realm. Always returns 204 to prevent email enumeration.",
    params(
        ("realm_name" = String, Path, description = "The realm name"),
    ),
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Request processed (email sent if user exists)", body = ForgotPasswordResponse),
        (status = 400, description = "Bad Request", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
pub async fn forgot_password(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<ForgotPasswordRequest>,
) -> Result<Response<ForgotPasswordResponse>, ApiError> {
    let base_url = state.args.webapp_url.trim_end_matches('/').to_string();
    let session_code = cookie
        .get("FERRISKEY_SESSION")
        .map(|c| c.value().to_string());

    state
        .service
        .request_password_reset(RequestPasswordResetInput {
            realm_name,
            email: payload.email,
            base_url,
            session_code,
        })
        .await?;

    Ok(Response::OK(ForgotPasswordResponse))
}

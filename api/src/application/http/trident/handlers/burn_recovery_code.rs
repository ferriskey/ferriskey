use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{BurnRecoveryCodeInput, TridentService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct BurnRecoveryCodeRequest {
    recovery_code: String,
    recovery_code_format: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct BurnRecoveryCodeResponse {
    login_url: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/burn-recovery-code",
    tag = "auth",
    summary = "Burn a recovery code to authenticate",
    description = "Using a recovery code allows a user to bypass a MFA challenge",
    request_body = BurnRecoveryCodeRequest,
    responses(
        (status = 200, body = BurnRecoveryCodeResponse),
        (status = 400, body = String)
    )
)]
pub async fn burn_recovery_code(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<BurnRecoveryCodeRequest>,
) -> Result<Response<BurnRecoveryCodeResponse>, ApiError> {
    let session_code = cookie.get("FERRISKEY_SESSION").unwrap();
    let session_code = session_code.value().to_string();

    let result = state
        .service
        .burn_recovery_code(
            identity,
            BurnRecoveryCodeInput {
                session_code,
                format: payload.recovery_code_format,
                code: payload.recovery_code,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(BurnRecoveryCodeResponse {
        login_url: result.login_url,
    }))
}

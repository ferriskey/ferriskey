use axum::{Extension, extract::State};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{GenerateRecoveryCodeInput, TridentService},
};
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

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct GenerateRecoveryCodesRequest {
    amount: u8,
    code_format: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct GenerateRecoveryCodesResponse {
    codes: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/login-actions/generate-recovery-codes",
    tag = "auth",
    summary = "Generate recovery codes",
    description = "Generates recovery codes that allows the user to bypass a MFA challenge",
    request_body = GenerateRecoveryCodesRequest,
    responses(
        (status = 200, description = "Successfully generated recovery codes", body = GenerateRecoveryCodesResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 404, description = "Session not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn generate_recovery_codes(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<GenerateRecoveryCodesRequest>,
) -> Result<Response<GenerateRecoveryCodesResponse>, ApiError> {
    let result = state
        .service
        .generate_recovery_code(
            identity,
            GenerateRecoveryCodeInput {
                amount: payload.amount,
                format: payload.code_format,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GenerateRecoveryCodesResponse {
        codes: result.codes,
    }))
}

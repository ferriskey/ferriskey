use axum::extract::{Path, State};
use ferriskey_core::domain::email_verification::ports::{
    EmailVerificationService, VerifyEmailResult,
};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct VerifyEmailRequest {
    /// Email verification token
    #[validate(length(min = 1))]
    pub token: String,
}

/// POST /realms/{realm_name}/login-actions/verify-email
#[utoipa::path(
    post,
    path = "/login-actions/verify-email",
    tag = "auth",
    summary = "Verify email address",
    description = "Verify a user's email address using the token from the verification email.",
    request_body = VerifyEmailRequest,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Email verified successfully", body = VerifyEmailResult),
        (status = 400, description = "Invalid or expired token", body = ApiErrorResponse),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn verify_email_handler(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<VerifyEmailRequest>,
) -> Result<Response<VerifyEmailResult>, ApiError> {
    let result = state
        .service
        .email_verification_service
        .verify_email(realm_name, payload.token)
        .await?;

    Ok(Response::OK(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_verify_email_request_deserialization() {
        let request = VerifyEmailRequest {
            token: "test-token".to_string(),
        };
        assert_eq!(request.token, "test-token");
    }

    #[test]
    fn test_verify_email_result_serialization() {
        let user_id = Uuid::new_v4();
        let result = VerifyEmailResult {
            user_id,
            verified: true,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains(&user_id.to_string()));
        assert!(json.contains("verified"));
        assert!(json.contains("true"));
    }
}

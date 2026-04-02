use axum::extract::{Path, Query, State};
use ferriskey_core::domain::email_verification::ports::{
    EmailVerificationService, VerifyEmailResult,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::application::http::server::{api_entities::api_error::ApiError, app_state::AppState};

#[derive(Debug, Deserialize, IntoParams)]
pub struct VerifyEmailQuery {
    /// Email verification token
    pub token: String,
}

/// GET /realms/{realm_name}/login-actions/verify-email?token=xxx
#[utoipa::path(
    get,
    path = "/realms/{realm_name}/login-actions/verify-email",
    tag = "auth",
    summary = "Verify email address",
    description = "Verify a user's email address using the token from the verification email",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        VerifyEmailQuery,
    ),
    responses(
        (status = 200, description = "Email verified successfully", body = VerifyEmailResult),
        (status = 400, description = "Invalid or expired token"),
    ),
)]
pub async fn verify_email_handler(
    Path(_realm_name): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<VerifyEmailQuery>,
) -> Result<axum::Json<VerifyEmailResult>, ApiError> {
    let result = state
        .service
        .email_verification_service
        .verify_email(query.token)
        .await?;

    Ok(axum::Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_verify_email_query_deserialization() {
        let query = VerifyEmailQuery {
            token: "test-token".to_string(),
        };
        assert_eq!(query.token, "test-token");
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

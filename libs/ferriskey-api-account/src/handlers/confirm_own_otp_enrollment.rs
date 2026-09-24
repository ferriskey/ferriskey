use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::account_security::entities::ConfirmOwnOtpEnrollmentInput;
use ferriskey_core::domain::account_security::ports::AccountSecurityService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::session::CallerSession;
use crate::validators::ConfirmOwnOtpEnrollmentValidator;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ConfirmOtpEnrollmentResponse {
    pub message: String,
}

#[utoipa::path(
    put,
    path = "/me/mfa/otp",
    tag = "account",
    summary = "Confirm an authenticator enrolment",
    description = "Verifies a code against the secret recorded by `POST /mfa/otp` and promotes it to a real credential, superseding whatever authenticator the account had. Requires an elevation proved with a password: the code alone must not be able to replace the factor it proves.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body(
        content = ConfirmOwnOtpEnrollmentValidator,
        description = "The code from the authenticator, an optional label, and a live elevation",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Authenticator enrolled", body = ConfirmOtpEnrollmentResponse),
        (status = 400, description = "Invalid request data, or the code does not match the pending enrolment", body = ApiErrorResponse),
        (status = 403, description = "No live elevation, or the elevation was not proved with a password", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn confirm_own_otp_enrollment(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CallerSession(session_id): CallerSession,
    ValidateJson(payload): ValidateJson<ConfirmOwnOtpEnrollmentValidator>,
) -> Result<Response<ConfirmOtpEnrollmentResponse>, ApiError> {
    state
        .service
        .confirm_own_otp_enrollment(
            identity,
            ConfirmOwnOtpEnrollmentInput {
                realm_name,
                session_id,
                elevation_id: payload.elevation_id,
                code: payload.code,
                label: payload.label,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ConfirmOtpEnrollmentResponse {
        message: "Authenticator enrolled".to_string(),
    }))
}

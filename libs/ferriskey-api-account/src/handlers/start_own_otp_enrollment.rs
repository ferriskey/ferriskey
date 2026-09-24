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
    url::FullUrl,
};
use ferriskey_core::domain::account_security::entities::StartOwnOtpEnrollmentInput;
use ferriskey_core::domain::account_security::ports::AccountSecurityService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::session::CallerSession;
use crate::validators::StartOwnOtpEnrollmentValidator;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct StartOtpEnrollmentResponse {
    pub secret: String,
    pub otpauth_uri: String,
}

#[utoipa::path(
    post,
    path = "/me/mfa/otp",
    tag = "account",
    summary = "Start enrolling an authenticator",
    description = "Generates a TOTP secret, records it server-side under a short TTL, and returns it with its otpauth URI so the caller can scan it. The secret is never taken back from the caller: `PUT /mfa/otp` reads it from this side of the wire. Requires an elevation proved with a password, refused here rather than at the confirm so the caller is not sent to scan a code they will not be allowed to submit.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body(
        content = StartOwnOtpEnrollmentValidator,
        description = "A live elevation",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Enrolment started", body = StartOtpEnrollmentResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 403, description = "No live elevation for this caller and session", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn start_own_otp_enrollment(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CallerSession(session_id): CallerSession,
    FullUrl(_, base_url): FullUrl,
    ValidateJson(payload): ValidateJson<StartOwnOtpEnrollmentValidator>,
) -> Result<Response<StartOtpEnrollmentResponse>, ApiError> {
    let issuer = format!("{base_url}/realms/{realm_name}");

    let enrollment = state
        .service
        .start_own_otp_enrollment(
            identity,
            StartOwnOtpEnrollmentInput {
                realm_name: realm_name.clone(),
                session_id,
                elevation_id: payload.elevation_id,
                issuer,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(StartOtpEnrollmentResponse {
        secret: enrollment.secret,
        otpauth_uri: enrollment.otpauth_uri,
    }))
}

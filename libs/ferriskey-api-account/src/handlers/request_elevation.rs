use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorBody, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::account_security::entities::{ElevationProof, RequestElevationInput};
use ferriskey_core::domain::account_security::ports::AccountSecurityService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::session::CallerSession;
use crate::validators::RequestElevationValidator;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ElevationResponse {
    pub elevation_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/me/reauthenticate",
    tag = "account",
    summary = "Re-authenticate to unlock sensitive account operations",
    description = "Verifies a credential the caller already holds and returns a short-lived elevation bound to the caller's session. Every sensitive account operation requires one. Supply exactly one of `password` or `otp_code`. Anything that adds, replaces or removes a sign-in credential requires the elevation to have been proved with a password — a second factor must not be able to rotate the factors, including itself. Changing the password is the exception: it verifies the current password in the body, which is the same proof.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body(
        content = RequestElevationValidator,
        description = "The credential proving the caller is still who they claim",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Elevation granted", body = ElevationResponse),
        (status = 400, description = "Invalid request data, or not exactly one of password and otp_code", body = ApiErrorResponse),
        (status = 401, description = "The proof did not check out, the account is locked, or the token carries no session", body = ApiErrorResponse),
        (status = 403, description = "The caller is not a regular user", body = ApiErrorResponse),
        (status = 404, description = "No such realm", body = ApiErrorResponse),
        (status = 409, description = "The account signs in through an external directory and has no local password", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn request_elevation(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CallerSession(session_id): CallerSession,
    ValidateJson(payload): ValidateJson<RequestElevationValidator>,
) -> Result<Response<ElevationResponse>, ApiError> {
    let proof = match (payload.password, payload.otp_code) {
        (Some(password), None) => ElevationProof::Password(password),
        (None, Some(code)) => ElevationProof::Otp(code),
        _ => {
            return Err(ApiError::BadRequest(ApiErrorBody::new(
                "Supply exactly one of password or otp_code",
                "invalid_elevation_proof",
            )));
        }
    };

    let elevation = state
        .service
        .request_elevation(
            identity,
            RequestElevationInput {
                realm_name,
                session_id,
                proof,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ElevationResponse {
        elevation_id: elevation.elevation_id,
        expires_at: elevation.expires_at,
    }))
}

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
use ferriskey_core::domain::account_security::entities::DisableOwnOtpInput;
use ferriskey_core::domain::account_security::ports::AccountSecurityService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::session::CallerSession;
use crate::validators::ElevationOnlyValidator;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DisableOtpResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/me/mfa/otp",
    tag = "account",
    summary = "Remove the caller's authenticator",
    description = "Refused when it would leave the account with no way to sign in, or without the second factor the realm or one of the caller's roles requires. Requires an elevation proved with a password.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body(
        content = ElevationOnlyValidator,
        description = "A live elevation",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Authenticator removed", body = DisableOtpResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 403, description = "No live elevation, or the elevation was not proved with a password", body = ApiErrorResponse),
        (status = 409, description = "The removal would lock the account out or break its MFA requirement", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn disable_own_otp(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CallerSession(session_id): CallerSession,
    ValidateJson(payload): ValidateJson<ElevationOnlyValidator>,
) -> Result<Response<DisableOtpResponse>, ApiError> {
    state
        .service
        .disable_own_otp(
            identity,
            DisableOwnOtpInput {
                realm_name,
                session_id,
                elevation_id: payload.elevation_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DisableOtpResponse {
        message: "Authenticator removed".to_string(),
    }))
}

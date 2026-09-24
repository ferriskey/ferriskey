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
use ferriskey_core::domain::account_security::entities::ChangeOwnPasswordInput;
use ferriskey_core::domain::account_security::ports::AccountSecurityService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::session::CallerSession;
use crate::validators::ChangeOwnPasswordValidator;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ChangeOwnPasswordResponse {
    pub message: String,
}

#[utoipa::path(
    put,
    path = "/me/password",
    tag = "account",
    summary = "Change the caller's own password",
    description = "Verifies the current password, applies the realm password policy, and replaces the credential. Every other session of this account is revoked; the caller's own session survives. Any live elevation is accepted here, including one proved with a TOTP code: the current password supplied in the body is itself the primary proof, so demanding a password-proved elevation on top would only make the caller type it twice.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body(
        content = ChangeOwnPasswordValidator,
        description = "The current password, the new one, and a live elevation",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Password changed", body = ChangeOwnPasswordResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "The current password did not check out, or the account is locked", body = ApiErrorResponse),
        (status = 403, description = "No live elevation for this caller and session", body = ApiErrorResponse),
        (status = 404, description = "No such realm", body = ApiErrorResponse),
        (status = 409, description = "The account signs in through an external directory and has no local password", body = ApiErrorResponse),
        (status = 422, description = "The new password violates the realm password policy", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn change_own_password(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CallerSession(session_id): CallerSession,
    ValidateJson(payload): ValidateJson<ChangeOwnPasswordValidator>,
) -> Result<Response<ChangeOwnPasswordResponse>, ApiError> {
    state
        .service
        .change_own_password(
            identity,
            ChangeOwnPasswordInput {
                realm_name,
                session_id,
                elevation_id: payload.elevation_id,
                current_password: payload.current_password,
                new_password: payload.new_password,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ChangeOwnPasswordResponse {
        message: "Password changed".to_string(),
    }))
}

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
use ferriskey_core::domain::account_security::entities::ConfirmOwnPasskeyRegistrationInput;
use ferriskey_core::domain::account_security::ports::AccountSecurityService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::session::CallerSession;
use crate::validators::{ConfirmOwnPasskeyRegistrationValidator, webauthn_rp_info_from_webapp_url};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ConfirmPasskeyResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/me/passkeys",
    tag = "account",
    summary = "Finish registering a passkey",
    description = "Answers the challenge issued by `POST /passkeys/options` and stores the resulting passkey on the caller's own account. Requires an elevation proved with a password, for the same reason as the challenge itself.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body(
        content = ConfirmOwnPasskeyRegistrationValidator,
        description = "The authenticator's response and a live elevation",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Passkey registered", body = ConfirmPasskeyResponse),
        (status = 400, description = "Invalid request data, or no live registration challenge to answer", body = ApiErrorResponse),
        (status = 401, description = "The challenge could not be verified, or the account is locked", body = ApiErrorResponse),
        (status = 403, description = "No live elevation for this caller and session, or the elevation was not proved with a password", body = ApiErrorResponse),
        (status = 404, description = "No such realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn confirm_own_passkey_registration(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CallerSession(session_id): CallerSession,
    ValidateJson(payload): ValidateJson<ConfirmOwnPasskeyRegistrationValidator>,
) -> Result<Response<ConfirmPasskeyResponse>, ApiError> {
    let rp_info = webauthn_rp_info_from_webapp_url(&state.args.webapp_url);

    state
        .service
        .confirm_own_passkey_registration(
            identity,
            ConfirmOwnPasskeyRegistrationInput {
                realm_name,
                session_id,
                elevation_id: payload.elevation_id,
                rp_info,
                credential: payload.credential,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ConfirmPasskeyResponse {
        message: "Passkey registered".to_string(),
    }))
}

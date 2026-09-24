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
use ferriskey_core::domain::account_security::entities::DeleteOwnPasskeyInput;
use ferriskey_core::domain::account_security::ports::AccountSecurityService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::session::CallerSession;
use crate::validators::ElevationOnlyValidator;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeletePasskeyResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/me/passkeys/{credential_id}",
    tag = "account",
    summary = "Remove one of the caller's passkeys",
    description = "Refused when it would leave the account with no way to sign in, or without the second factor the realm or one of the caller's roles requires. A credential that is not a passkey, or belongs to someone else, answers not found. Requires an elevation proved with a password.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("credential_id" = Uuid, Path, description = "Identifier of the passkey to remove"),
    ),
    request_body(
        content = ElevationOnlyValidator,
        description = "A live elevation",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Passkey removed", body = DeletePasskeyResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 403, description = "No live elevation, or the elevation was not proved with a password", body = ApiErrorResponse),
        (status = 404, description = "No such passkey on this account", body = ApiErrorResponse),
        (status = 409, description = "The removal would lock the account out or break its MFA requirement", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn delete_own_passkey(
    Path((realm_name, credential_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CallerSession(session_id): CallerSession,
    ValidateJson(payload): ValidateJson<ElevationOnlyValidator>,
) -> Result<Response<DeletePasskeyResponse>, ApiError> {
    state
        .service
        .delete_own_passkey(
            identity,
            DeleteOwnPasskeyInput {
                realm_name,
                session_id,
                elevation_id: payload.elevation_id,
                credential_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeletePasskeyResponse {
        message: "Passkey removed".to_string(),
    }))
}

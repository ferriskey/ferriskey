use axum::{
    Extension,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::account_security::entities::ListOwnCredentialsInput;
use ferriskey_core::domain::account_security::ports::AccountSecurityService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct OwnCredentialDto {
    pub id: Uuid,
    pub credential_type: String,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct OwnCredentialsResponse {
    pub data: Vec<OwnCredentialDto>,
}

#[utoipa::path(
    get,
    path = "/me/credentials",
    tag = "account",
    summary = "List the caller's own sign-in credentials",
    description = "Returns what the caller can sign in with: their password, their authenticator, and each of their passkeys. Never returns secret material. The user id is taken from the access token, never from the URL.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Credentials listed", body = OwnCredentialsResponse),
        (status = 403, description = "The caller is not a regular user", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn list_own_credentials(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<OwnCredentialsResponse>, ApiError> {
    let credentials = state
        .service
        .list_own_credentials(identity, ListOwnCredentialsInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    let data = credentials
        .into_iter()
        .map(|credential| OwnCredentialDto {
            id: credential.id,
            credential_type: credential.credential_type.to_string(),
            label: credential.label,
            created_at: credential.created_at,
        })
        .collect();

    Ok(Response::OK(OwnCredentialsResponse { data }))
}

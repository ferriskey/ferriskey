use crate::validators::ImportPasswordCredentialValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::ImportPasswordCredentialInput;
use ferriskey_core::domain::user::ports::UserService;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ImportPasswordCredentialResponse {
    pub message: String,
    pub user_id: Uuid,
    pub realm_name: String,
}

#[utoipa::path(
    post,
    path = "/{user_id}/credentials/import",
    tag = "user",
    summary = "Import a pre-hashed password",
    description = "Stores a password hash produced by another identity provider as the user's password credential. Supported algorithms are bcrypt ($2a$, $2b$, $2y$) and argon2 (PHC string). The hash is re-encoded as argon2id on the user's first successful login.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    request_body(
        content = ImportPasswordCredentialValidator,
        description = "Password hash to import",
        content_type = "application/json",
    ),
    responses(
        (status = 201, description = "Password hash imported", body = ImportPasswordCredentialResponse),
        (status = 400, description = "Invalid request body", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "User not found", body = ApiErrorResponse),
        (status = 409, description = "User already has a password credential", body = ApiErrorResponse),
        (status = 422, description = "Unsupported or malformed password hash", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn import_password_credential(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ImportPasswordCredentialValidator>,
) -> Result<Response<ImportPasswordCredentialResponse>, ApiError> {
    info!(
        "import password credential for user {user_id} in realm {realm_name} ({})",
        payload.algorithm
    );

    state
        .service
        .import_password_credential(
            identity,
            ImportPasswordCredentialInput {
                realm_name: realm_name.clone(),
                user_id,
                algorithm: payload.algorithm,
                secret_data: payload.secret_data,
                hash_iterations: payload.hash_iterations,
                salt: payload.salt,
                temporary: payload.temporary,
            },
        )
        .await
        .map_err(|e| {
            warn!("failed to import password for user {user_id} in realm {realm_name}: {e:?}");
            ApiError::from(e)
        })?;

    Ok(Response::Created(ImportPasswordCredentialResponse {
        message: "Password imported successfully".to_string(),
        user_id,
        realm_name,
    }))
}

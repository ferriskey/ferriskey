use std::sync::Arc;

use axum::{Extension, http::StatusCode};
use axum_macros::TypedPath;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::api_success::ApiSuccess;
use crate::application::http::user::validators::ResetPasswordValidator;
use crate::domain::credential::ports::credential_service::CredentialService;

#[derive(Deserialize, TypedPath)]
#[typed_path("/realms/{realm_name}/users/{user_id}/reset-password")]
pub struct ResetPasswordRoute {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[utoipa::path(
    put,
    path = "/{user_id}/reset-password",
    tag = "user",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
)]
pub async fn reset_password<C: CredentialService>(
    ResetPasswordRoute {
        user_id,
        realm_name,
    }: ResetPasswordRoute,
    Extension(credential_service): Extension<Arc<C>>,
    ValidateJson(payload): ValidateJson<ResetPasswordValidator>,
) -> Result<ApiSuccess<String>, ApiError> {
    info!(
        "reset password for user {:} in realm {:}",
        user_id, realm_name
    );
    credential_service
        .reset_password(user_id, payload.value)
        .await
        .map_err(|_| ApiError::InternalServerError("Internal server error".to_string()))?;
    Ok(ApiSuccess::new(
        StatusCode::OK,
        "Password reset successfully".to_string(),
    ))
}

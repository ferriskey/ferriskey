use std::sync::Arc;

use axum::{Extension, http::StatusCode};
use axum_macros::TypedPath;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    application::http::{
        server::{
            errors::{ApiError, ValidateJson},
            handlers::ApiSuccess,
        },
        user::validators::ResetPasswordValidator,
    },
    domain::credential::ports::CredentialService,
};

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
    credential_service
        .reset_password(user_id, payload.value)
        .await
        .map_err(|_| ApiError::InternalServerError("Internal server error".to_string()))?;
    Ok(ApiSuccess::new(
        StatusCode::OK,
        "Password reset successfully".to_string(),
    ))
}

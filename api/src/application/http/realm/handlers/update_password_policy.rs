use crate::application::http::server::app_state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use crate::application::http::errors::error::ApiError;
use ferriskey_domain::realm::{PasswordPolicy, UpdatePasswordPolicy};

/// Update the password policy for a realm
#[utoipa::path(
    put,
    path = "/realms/{realm_name}/password-policy",
    request_body = UpdatePasswordPolicy,
    responses(
        (status = 200, description = "Password policy updated", body = PasswordPolicy),
        (status = 404, description = "Realm not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name")
    ),
    tag = "Realm"
)]
pub async fn update_password_policy(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Json(update): Json<UpdatePasswordPolicy>,
) -> Result<Json<PasswordPolicy>, ApiError> {
    let realm = state
        .application_service
        .realm_service
        .realm_repository
        .get_by_name(realm_name)
        .await
        .map_err(|_| ApiError::InternalServerError)?
        .ok_or(ApiError::NotFound("Realm not found".to_string()))?;

    let policy = state
        .application_service
        .password_policy_service
        .update_policy(realm.id.into(), update)
        .await?;

    Ok(Json(policy))
}

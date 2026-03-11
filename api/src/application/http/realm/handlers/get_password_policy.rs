use crate::application::http::server::app_state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use crate::application::http::errors::error::ApiError;
use ferriskey_domain::realm::PasswordPolicy;

/// Get the password policy for a realm
#[utoipa::path(
    get,
    path = "/realms/{realm_name}/password-policy",
    responses(
        (status = 200, description = "Password policy found", body = PasswordPolicy),
        (status = 404, description = "Realm not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name")
    ),
    tag = "Realm"
)]
pub async fn get_password_policy(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
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
        .get_policy(realm.id.into())
        .await?;

    Ok(Json(policy))
}

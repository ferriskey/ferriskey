use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::app_state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use ferriskey_core::domain::password_policy::entities::{PasswordPolicy, UpdatePasswordPolicy};
use ferriskey_core::domain::password_policy::ports::PasswordPolicyService;
use ferriskey_core::domain::realm::ports::RealmService;

/// Update the password policy for a realm
#[utoipa::path(
    put,
    path = "/{realm_name}/password-policy",
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
    let realm_id = state
        .service
        .get_realm_id_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let policy = state
        .service
        .update_policy(realm_id.into(), update)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(policy))
}

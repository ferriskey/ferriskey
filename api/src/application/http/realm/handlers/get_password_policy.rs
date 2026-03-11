use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::app_state::AppState;
use axum::{
    extract::{Path, State},
    Extension,
    Json,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::password_policy::entities::PasswordPolicy;
use ferriskey_core::domain::password_policy::ports::PasswordPolicyService;
use ferriskey_core::domain::realm::ports::{GetRealmInput, RealmService};

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
    Extension(identity): Extension<Identity>,
) -> Result<Json<PasswordPolicy>, ApiError> {
    let realm = state
        .service
        .get_realm_by_name(identity.clone(), GetRealmInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    let policy = state
        .service
        .get_policy(realm.id.into())
        .await
        .map_err(ApiError::from)?;

    Ok(Json(policy))
}

use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{
    extract::{Path, State},
    Extension,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    password_policy::{PasswordPolicy, PasswordPolicyService, UpdatePasswordPolicy},
    realm::{ports::{GetRealmInput, RealmService}, entities::Realm},
};

/// Get the password policy for a realm
#[utoipa::path(
    get,
    path = "/{realm_name}/password-policy",
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
) -> Result<Response<PasswordPolicy>, ApiError> {
    let realm: Realm = state
        .service
        .realm_service
        .get_realm_by_name(identity, GetRealmInput { realm_name })
        .await
        .map_err(|e: ferriskey_core::domain::common::entities::app_errors::CoreError| ApiError::from(e))?;

    let policy = state
        .service
        .password_policy_service
        .get_policy(realm.id.into())
        .await?;

    Ok(Response::OK(policy))
}

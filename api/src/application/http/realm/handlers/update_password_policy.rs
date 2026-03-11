use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{
    extract::{Path, State},
    Extension,
    Json,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    password_policy::{PasswordPolicy, PasswordPolicyService, UpdatePasswordPolicy},
    realm::{ports::{GetRealmInput, RealmService}, entities::Realm},
};

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
    Extension(identity): Extension<Identity>,
    Json(update): Json<UpdatePasswordPolicy>,
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
        .update_policy(realm.id.into(), update)
        .await?;

    Ok(Response::OK(policy))
}

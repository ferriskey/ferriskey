use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    password_policy::entity::PasswordPolicy,
    realm::ports::{GetRealmInput, RealmService},
};

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

#[utoipa::path(
    get,
    path = "/{realm_name}/password-policy",
    tag = "realm",
    summary = "Get password policy for a realm",
    description = "Retrieves the password policy for the specified realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Password policy retrieved successfully", body = PasswordPolicy),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_password_policy(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<PasswordPolicy>, ApiError> {
    // First get the realm to get its ID
    let realm = state
        .service
        .get_realm_by_name(identity.clone(), GetRealmInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    let policy = state
        .service
        .get_password_policy(identity, realm.id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(policy))
}

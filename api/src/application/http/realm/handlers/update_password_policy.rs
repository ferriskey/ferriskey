use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    password_policy::entity::{PasswordPolicy, UpdatePasswordPolicy},
    realm::ports::{GetRealmInput, RealmService},
};

use crate::application::http::{
    realm::validators::UpdatePasswordPolicyValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[utoipa::path(
    put,
    path = "/{realm_name}/password-policy",
    tag = "realm",
    summary = "Update password policy for a realm",
    description = "Updates the password policy for the specified realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = UpdatePasswordPolicyValidator,
    responses(
        (status = 200, description = "Password policy updated successfully", body = PasswordPolicy),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_password_policy(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdatePasswordPolicyValidator>,
) -> Result<Response<PasswordPolicy>, ApiError> {
    // First get the realm to get its ID
    let realm = state
        .service
        .get_realm_by_name(identity.clone(), GetRealmInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    let update = UpdatePasswordPolicy {
        min_length: payload.min_length,
        require_uppercase: payload.require_uppercase,
        require_lowercase: payload.require_lowercase,
        require_number: payload.require_number,
        require_special: payload.require_special,
        max_age_days: payload.max_age_days,
    };

    let policy = state
        .service
        .update_password_policy(identity, realm.id, update)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(policy))
}

use axum::extract::{Path, State};
use ferriskey_core::domain::password_policy::entity::PasswordPolicy;
use serde::Serialize;
use utoipa::ToSchema;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

/// A trimmed, anonymous-safe view of the realm password policy.
#[derive(Debug, Serialize, ToSchema)]
pub struct PublicPasswordPolicy {
    pub min_length: i32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_special: bool,
}

impl From<PasswordPolicy> for PublicPasswordPolicy {
    fn from(p: PasswordPolicy) -> Self {
        Self {
            min_length: p.min_length,
            require_uppercase: p.require_uppercase,
            require_lowercase: p.require_lowercase,
            require_number: p.require_number,
            require_special: p.require_special,
        }
    }
}

#[utoipa::path(
    get,
    path = "/{realm_name}/password-policy/public",
    tag = "realm",
    summary = "Get public password policy for a realm",
    description = "Returns the password policy rules for the specified realm. No authentication required.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Password policy retrieved successfully", body = PublicPasswordPolicy),
        (status = 401, description = "Invalid realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_public_password_policy(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Response<PublicPasswordPolicy>, ApiError> {
    let policy = state
        .service
        .get_public_password_policy(realm_name)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(PublicPasswordPolicy::from(policy)))
}

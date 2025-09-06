use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{TridentService, UpdatePasswordInput},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdatePasswordRequest {
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
pub struct UpdatePasswordResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/update-password",
    tag = "auth",
    summary = "Update Password",
    request_body = UpdatePasswordRequest,
    responses(
        (status = 200, description = "Password updated successfully", body = UpdatePasswordResponse),
    )
)]
pub async fn update_password(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdatePasswordRequest>,
) -> Result<Response<UpdatePasswordResponse>, ApiError> {
    state
        .service
        .update_password(
            identity,
            UpdatePasswordInput {
                value: payload.value,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UpdatePasswordResponse {
        message: "Password updated successfully".to_string(),
    }))
}

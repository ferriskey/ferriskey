use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{TridentService, UpdatePasswordInput},
    user::entities::RequiredAction,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
    login_step::{LoginActionStep, LoginStepReply},
    url::{FullUrl, WebappUrl},
};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdatePasswordRequest {
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
pub struct UpdatePasswordResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub required_actions: Vec<RequiredAction>,
}

#[utoipa::path(
    post,
    path = "/login-actions/update-password",
    tag = "auth",
    summary = "Update Password",
    request_body = UpdatePasswordRequest,
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Password updated successfully", body = UpdatePasswordResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 403, description = "Identity not authorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn update_password(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    FullUrl(_, base_url): FullUrl,
    WebappUrl(webapp_url): WebappUrl,
    Extension(step): Extension<LoginActionStep>,
    ValidateJson(payload): ValidateJson<UpdatePasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .service
        .validate_password_policy(realm_name.clone(), &payload.value)
        .await?;

    let progress = state
        .service
        .update_password(
            identity,
            UpdatePasswordInput {
                value: payload.value,
                realm_name,
                session_code: step.auth_session_id.to_string(),
                second_factor_verified: step.second_factor_verified,
                webapp_base_url: webapp_url,
            },
        )
        .await
        .map_err(ApiError::from)?;

    let reply = LoginStepReply::new(progress, base_url.starts_with("https"))?;

    Ok((
        reply.headers,
        reply.continues,
        Response::OK(UpdatePasswordResponse {
            message: "Password updated successfully".to_string(),
            url: reply.url,
            required_actions: reply.required_actions,
        }),
    ))
}

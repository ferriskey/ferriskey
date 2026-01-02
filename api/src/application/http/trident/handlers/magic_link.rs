use axum::{extract::State, response::Response};
use serde::{Deserialize, Serialize};
use tracing::debug;
use utoipa::ToSchema;
use validator::Validate;

use crate::application::http::server::{
    api_entities::api_error::{ApiError, ValidateJson},
    app_state::AppState,
};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct MagicLinkRequest {
    #[serde(default)]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct MagicLinkVerifyRequest {
    pub token: String,
}

pub async fn send_magic_link(
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<MagicLinkRequest>,
) -> Result<Response, ApiError> {
    // if ok email has been verified then
    // Console log the token
    debug!(
        "pretend to use variables to avoid pre commit problems {:?} {:?}",
        state.args, payload
    );
    Ok(Response::default())
}

pub async fn verify_magic_link(
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<MagicLinkVerifyRequest>,
) -> Result<Response, ApiError> {
    // If ok token has been verified then
    // call authenticate method
    debug!(
        "pretend to use variables to avoid pre commit problems {:?} {:?}",
        state.args, payload
    );
    Ok(Response::default())
}

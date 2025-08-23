use axum::extract::State;
use serde::Serialize;
use utoipa::ToSchema;

use super::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct GetConfigResponse {
    pub app_version: String,
    pub environment: String,
}

pub async fn get_config(
    State(state): State<AppState>,
) -> Result<Response<GetConfigResponse>, ApiError> {
    Ok(Response::OK(GetConfigResponse {
        environment: state.args.env.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

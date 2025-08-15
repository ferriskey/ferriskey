use axum::extract::State;
use axum_macros::TypedPath;
use serde::Serialize;
use utoipa::ToSchema;

use super::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[derive(TypedPath)]
#[typed_path("/config")]
pub struct GetConfigRoute;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct GetConfigResponse {
    pub app_version: String,
    pub environment: String,
}

pub async fn get_config(
    GetConfigRoute: GetConfigRoute,
    State(state): State<AppState>,
) -> Result<Response<GetConfigResponse>, ApiError> {
    let environment = state.env.env.clone();
    let environment = environment.to_string();

    let app_version = env!("CARGO_PKG_VERSION").to_string();

    let response = GetConfigResponse {
        environment,
        app_version,
    };

    Ok(Response::OK(response))
}

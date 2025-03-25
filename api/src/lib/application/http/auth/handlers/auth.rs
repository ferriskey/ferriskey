use axum::{
    extract::Query,
    http::Response,
    response::{IntoResponse, Redirect},
};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::application::http::server::errors::{ApiError, ValidateJson};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AuthRequest {
    #[validate(length(min = 1, message = "response_type is required"))]
    #[serde(default)]
    pub response_type: String,
    #[validate(length(min = 1, message = "client_id is required"))]
    #[serde(default)]
    pub client_id: String,
    #[validate(length(min = 1, message = "redirect_uri is required"))]
    #[serde(default)]
    pub redirect_uri: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/protocol/openid-connect/auth")]
pub struct AuthRoute {
    pub realm_name: String,
}

pub async fn auth(
    AuthRoute { realm_name }: AuthRoute,
    Query(params): Query<AuthRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let login_url = format!(
        "http://localhost:5672/realms/{}/login?client_id={}&redirect_uri={}&state={}",
        realm_name,
        params.client_id,
        params.redirect_uri,
        params.state.unwrap_or_default()
    );

    Ok(Redirect::to(&login_url))
}

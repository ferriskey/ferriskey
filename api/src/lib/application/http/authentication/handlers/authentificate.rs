use axum::Extension;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use utoipa::ToSchema;
use validator::Validate;

use crate::application::http::authentication::validators::TokenRequestValidator;
use crate::application::http::server::errors::{ApiError, ValidateJson};
use crate::application::http::server::handlers::Response;
use crate::domain::authentication::entities::model::JwtToken;
use crate::domain::authentication::ports::AuthenticationService;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AuthenticateRequest {
    #[validate(length(min = 1, message = "username is required"))]
    #[serde(default)]
    pub username: String,

    #[validate(length(min = 1, message = "password is required"))]
    #[serde(default)]
    pub password: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/login-actions/authenticate")]
pub struct TokenRoute {
    realm_name: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/authenticate",
    tag = "auth",
    request_body = AuthenticateRequest,
    responses(
        (status = 200, body = JwtToken)
    )
)]
pub async fn exchange_token<A: AuthenticationService>(
    TokenRoute { realm_name }: TokenRoute,
    Extension(authentication_service): Extension<Arc<A>>,
    ValidateJson(payload): ValidateJson<AuthenticateRequest>,
) -> Result<Response<JwtToken>, ApiError> {
    todo!("implement authenticate request");
}

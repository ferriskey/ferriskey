use axum::Extension;
use axum_macros::TypedPath;
use serde::Deserialize;
use std::sync::Arc;

use crate::application::http::authentication::validators::TokenRequestValidator;
use crate::application::http::server::errors::{ApiError, ValidateJson};
use crate::application::http::server::handlers::Response;
use crate::domain::authentication::entities::model::JwtToken;
use crate::domain::authentication::ports::AuthenticationService;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/protocol/openid-connect/token")]
pub struct TokenRoute {
    realm_name: String,
}

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/token",
    tag = "auth",
    request_body = TokenRequestValidator,
    responses(
        (status = 200, body = JwtToken)
    )
)]
pub async fn exchange_token<A: AuthenticationService>(
    TokenRoute { realm_name }: TokenRoute,
    Extension(token_service): Extension<Arc<A>>,
    ValidateJson(payload): ValidateJson<TokenRequestValidator>,
) -> Result<Response<JwtToken>, ApiError> {
    token_service
        .authentificate(
            realm_name,
            payload.grant_type,
            payload.client_id,
            payload.code,
            payload.username,
            payload.password,
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

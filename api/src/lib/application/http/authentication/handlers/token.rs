use axum::Extension;
use axum_macros::TypedPath;
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use crate::application::http::authentication::validators::TokenRequestValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::domain::authentication::entities::jwt_token::JwtToken;
use crate::domain::authentication::ports::authentication::AuthenticationService;

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
    Extension(authentication_service): Extension<Arc<A>>,
    ValidateJson(payload): ValidateJson<TokenRequestValidator>,
) -> Result<Response<JwtToken>, ApiError> {
    info!("request login with \"{:?}\" grant_type", payload.grant_type);
    authentication_service
        .authentificate(
            realm_name,
            payload.grant_type,
            payload.client_id,
            payload.client_secret,
            payload.code,
            payload.username,
            payload.password,
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

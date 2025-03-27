use axum::Extension;
use axum_macros::TypedPath;
use serde::{Deserialize};
use std::sync::Arc;

use crate::application::http::authentication::validators::TokenRequestValidator;
use crate::application::http::server::errors::{ApiError, ValidateJson};
use crate::application::http::server::handlers::Response;
use crate::domain::authentication::entities::model::ExchangeToken;
use crate::domain::authentication::ports::AuthenticationService;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{name}/oauth2/token")]
pub struct TokenRoute {
    name: String,
}

#[utoipa::path(
    post,
    path = "/oauth2/token",
    tag = "auth",
    request_body = TokenRequestValidator,
    responses(
        (status = 200, body = ExchangeToken)
    )
)]
pub async fn exchange_token<A: AuthenticationService>(
    _: TokenRoute,
    Extension(token_service): Extension<Arc<A>>,
    ValidateJson(payload): ValidateJson<TokenRequestValidator>,
) -> Result<Response<ExchangeToken>, ApiError> {
    token_service
        .exchange_token(payload.grant_type, payload.client_id, payload.code)
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

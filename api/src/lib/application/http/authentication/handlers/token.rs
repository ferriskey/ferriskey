use axum::extract::State;
use axum_macros::TypedPath;
use serde::Deserialize;
use tracing::info;

use crate::application::http::authentication::validators::TokenRequestValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
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
pub async fn exchange_token(
    TokenRoute { realm_name }: TokenRoute,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<TokenRequestValidator>,
) -> Result<Response<JwtToken>, ApiError> {
    info!("request login with \"{:?}\" grant_type", payload.grant_type);
    state
        .authentication_service
        .authentificate(
            realm_name,
            payload.grant_type,
            payload.client_id,
            payload.client_secret,
            payload.code,
            payload.username,
            payload.password,
            payload.refresh_token,
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

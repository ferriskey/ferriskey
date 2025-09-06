use crate::application::http::authentication::validators::TokenRequestValidator;
use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::application::url::FullUrl;
use axum::{
    Form,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::entities::JwtToken;
use ferriskey_core::domain::authentication::{entities::ExchangeTokenInput, ports::AuthService};

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/token",
    tag = "auth",
    summary = "Exchange token",
    description = "Exchanges a token for a JWT token. This endpoint allows clients to exchange various types of tokens (like authorization codes, refresh tokens, etc.) for a JWT token.",
    request_body = TokenRequestValidator,
    responses(
        (status = 200, body = JwtToken)
    )
)]
pub async fn exchange_token(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    Form(payload): Form<TokenRequestValidator>,
) -> Result<Response<JwtToken>, ApiError> {
    state
        .service
        .exchange_token(ExchangeTokenInput {
            realm_name,
            client_id: payload.client_id,
            client_secret: payload.client_secret,
            code: payload.code,
            username: payload.username,
            password: payload.password,
            refresh_token: payload.refresh_token,
            base_url,
            grant_type: payload.grant_type,
        })
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

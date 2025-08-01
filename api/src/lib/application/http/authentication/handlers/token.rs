use crate::application::http::authentication::validators::TokenRequestValidator;
use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::application::url::FullUrl;
use axum::Form;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::authentication::use_cases::exchange_token_use_case::ExchangeTokenUseCaseParams;
use ferriskey_core::domain::authentication::entities::JwtToken;
use serde::Deserialize;

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
    FullUrl(_, base_url): FullUrl,
    Form(payload): Form<TokenRequestValidator>,
) -> Result<Response<JwtToken>, ApiError> {
    state
        .use_case_bundle
        .exchange_token_use_case
        .execute(ExchangeTokenUseCaseParams {
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

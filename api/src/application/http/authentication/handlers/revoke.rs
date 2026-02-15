use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ferriskey_core::domain::authentication::{ports::AuthService, value_objects::RevokeTokenInput};
use validator::Validate;

use crate::application::http::authentication::validators::RevokeTokenRequestValidator;
use crate::application::http::server::{api_entities::api_error::ApiError, app_state::AppState};

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/revoke",
    tag = "auth",
    summary = "Token revocation",
    description = "OAuth2 token revocation endpoint (RFC 7009). Revokes access or refresh tokens for the requesting client.",
    request_body = RevokeTokenRequestValidator,
    params(
      ("realm_name" = String, Path, description = "Realm name")
    ),
    responses(
        (status = 200, description = "Token revocation processed")
    )
)]
pub async fn revoke_token(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Form(payload): Form<RevokeTokenRequestValidator>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;

    state
        .service
        .revoke_token(RevokeTokenInput {
            realm_name,
            client_id: payload.client_id,
            token: payload.token,
            token_type_hint: payload.token_type_hint,
        })
        .await?;

    Ok(StatusCode::OK)
}

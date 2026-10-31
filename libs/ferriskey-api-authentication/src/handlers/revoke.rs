use axum::{
    Form,
    extract::{Path, State},
    http::HeaderMap,
};
use ferriskey_core::domain::authentication::{ports::AuthService, value_objects::RevokeTokenInput};
use validator::Validate;

use super::authorization_server_metadata::realm_issuer;
use crate::client_credentials::{ClientAuthParams, assertion_audiences, presented_client};
use crate::validators::RevokeTokenRequestValidator;
use ferriskey_api_core::url::FullUrl;
use ferriskey_api_core::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

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
    FullUrl(_, base_url): FullUrl,
    headers: HeaderMap,
    Form(payload): Form<RevokeTokenRequestValidator>,
) -> Result<Response<()>, ApiError> {
    payload.validate()?;

    let issuer = realm_issuer(&base_url, &state.args.server.root_path, &realm_name);
    let presented = presented_client(
        &headers,
        ClientAuthParams {
            client_id: payload.client_id.as_deref(),
            client_secret: payload.client_secret.as_deref(),
            client_assertion_type: payload.client_assertion_type.as_deref(),
            client_assertion: payload.client_assertion.as_deref(),
        },
        || assertion_audiences(&issuer, "/protocol/openid-connect/revoke"),
    )?;

    state
        .service
        .revoke_token(RevokeTokenInput {
            realm_name,
            client_id: presented.client_id,
            client_credential: presented.credential,
            token: payload.token,
            token_type_hint: payload.token_type_hint,
        })
        .await?;

    Ok(Response::OK(()))
}

use axum::{
    Form,
    extract::{Path, State},
    http::HeaderMap,
};
use ferriskey_core::domain::authentication::{ports::AuthService, value_objects::RevokeTokenInput};
use validator::Validate;

use super::authorization_server_metadata::realm_issuer;
use super::token::{record_failure, reserve_client_auth};
use crate::client_credentials::{ClientAuthParams, assertion_audiences, presented_client};
use crate::validators::RevokeTokenRequestValidator;
use ferriskey_api_core::request_context::RequestContext;
use ferriskey_api_core::url::FullUrl;
use ferriskey_api_core::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use ferriskey_core::domain::common::entities::app_errors::CoreError;

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
    context: RequestContext,
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

    let attempt = reserve_client_auth(
        &state,
        context.ip_address.as_deref(),
        &realm_name,
        &presented.client_id,
    )?;

    if let Err(error) = state
        .service
        .revoke_token(RevokeTokenInput {
            realm_name: realm_name.clone(),
            client_id: presented.client_id.clone(),
            client_credential: presented.credential,
            token: payload.token,
            token_type_hint: payload.token_type_hint,
        })
        .await
    {
        if matches!(
            error,
            CoreError::ClientAuthenticationFailed | CoreError::InvalidClientSecret
        ) {
            record_failure(&state, attempt, &realm_name, &presented.client_id).await;
        }
        return Err(error.into());
    }

    Ok(Response::OK(()))
}

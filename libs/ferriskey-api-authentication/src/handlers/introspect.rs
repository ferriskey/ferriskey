use axum::{
    Form,
    extract::{Path, State},
    http::HeaderMap,
};
use ferriskey_core::domain::authentication::{
    entities::TokenIntrospectionResponse, ports::AuthService, value_objects::IntrospectTokenInput,
};
use validator::Validate;

use super::authorization_server_metadata::realm_issuer;
use super::token::{record_failure, reserve_client_auth};
use crate::client_credentials::{ClientAuthParams, assertion_audiences, presented_client};
use crate::validators::IntrospectRequestValidator;
use ferriskey_api_core::api_entities::api_error::ApiErrorResponse;
use ferriskey_api_core::request_context::RequestContext;
use ferriskey_api_core::url::FullUrl;
use ferriskey_api_core::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use ferriskey_core::domain::authentication::client_authentication::ClientCredential;
use ferriskey_core::domain::common::entities::app_errors::CoreError;

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/token/introspect",
    tag = "auth",
    summary = "Token introspection",
    description = "OAuth2/OIDC Token Introspection (RFC 7662). Only confidential clients may call this endpoint, authenticated with their registered method: client_secret_basic, client_secret_post, client_secret_jwt or private_key_jwt. Authorization requires the caller's service account to have the role `introspect` (treated as the `introspect` scope).",
    request_body = IntrospectRequestValidator,
    params(
      ("realm_name" = String, Path, description = "Realm name")
    ),
    responses(
        (status = 200, body = TokenIntrospectionResponse),
        (status = 401, description = "Missing or invalid client credentials", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
pub async fn introspect_token(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    context: RequestContext,
    headers: HeaderMap,
    Form(payload): Form<IntrospectRequestValidator>,
) -> Result<Response<TokenIntrospectionResponse>, ApiError> {
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
        || assertion_audiences(&issuer, "/protocol/openid-connect/token/introspect"),
    )?;

    if presented.client_id.is_empty() || matches!(presented.credential, ClientCredential::None) {
        return Err(ApiError::Unauthorized(
            "Missing client authentication".into(),
        ));
    }

    let attempt = reserve_client_auth(
        &state,
        context.ip_address.as_deref(),
        &realm_name,
        &presented.client_id,
    )?;

    let response = match state
        .service
        .introspect_token(IntrospectTokenInput {
            realm_name: realm_name.clone(),
            client_id: presented.client_id.clone(),
            client_credential: presented.credential,
            token: payload.token,
            token_type_hint: payload.token_type_hint,
        })
        .await
    {
        Ok(response) => response,
        Err(error) => {
            if matches!(
                error,
                CoreError::ClientAuthenticationFailed | CoreError::InvalidClientSecret
            ) {
                record_failure(&state, attempt, &realm_name, &presented.client_id).await;
            }
            return Err(error.into());
        }
    };

    Ok(Response::OK(response))
}

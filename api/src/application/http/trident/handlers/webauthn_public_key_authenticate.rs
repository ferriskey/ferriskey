use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::{
        entities::{
            WebAuthnAuthenticationExtensionsClientOutputs,
            WebAuthnAuthenticatorAssertionResponseJSON, WebAuthnCredentialIdGroup,
        },
        ports::{TridentService, WebAuthnPublicKeyAuthenticateInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use validator::Validate;

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(rename_all="camelCase")]
pub struct AuthenticationAttemptRequest {
    pub id: String,
    pub raw_id: String,
    pub response: WebAuthnAuthenticatorAssertionResponseJSON,
    pub authenticator_attachment: String,
    pub client_extension_results: WebAuthnAuthenticationExtensionsClientOutputs,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct AuthenticationAttemptResponse {
    login_url: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/webauthn-public-key-authenticate",
    tag = "auth",
    summary = "Authenticate using webauthn",
    description = "Attempt authentication using a WebAuthnAssertionResponse payload for webauthn authentication. See https://w3c.github.io/webauthn/#dictdef-authenticationresponsejson and https://w3c.github.io/webauthn/#authenticatorassertionresponse",
    request_body = AuthenticationAttemptRequest,
    responses(
        (status = 200, body = AuthenticationAttemptResponse),
    )
)]
pub async fn webauthn_public_key_authenticate(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<AuthenticationAttemptRequest>,
) -> Result<Response<AuthenticationAttemptResponse>, ApiError> {
    let session_code = cookie.get("FERRISKEY_SESSION").unwrap();
    let session_code = session_code.value().to_string();
    let session_code = uuid::Uuid::parse_str(&session_code).map_err(|_| {
        ApiError::BadRequest("Failed to parse session code as a valid UUID".to_string())
    })?;

    let credential = WebAuthnCredentialIdGroup::decode_and_verify(payload.id, payload.raw_id)
        .map_err(|msg| ApiError::BadRequest(format!("Failed to parse credential id: {msg}")))?;

    let response = payload
        .response
        .decode()
        .map_err(|msg| ApiError::BadRequest(format!("Failed to parse response object: {msg}")))?;

    let output = state
        .service
        .webauthn_public_key_authenticate(
            identity,
            WebAuthnPublicKeyAuthenticateInput {
                session_code,
                credential,
                response,
                typ: payload.typ,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(AuthenticationAttemptResponse {
        login_url: output.login_url,
    }))
}

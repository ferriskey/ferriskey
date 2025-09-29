use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use axum::{Extension, extract::State};
use ferriskey_core::domain::trident::entities::webauthn::{
    WebAuthnAuthenticationExtensionsClientOutputs, WebAuthnAuthenticatorAttestationResponse,
};
use ferriskey_core::domain::trident::ports::{TridentService, WebAuthnCredentialCreationInput};
use ferriskey_core::domain::{
    authentication::value_objects::Identity, trident::entities::WebAuthnCredentialId,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct VerifyWebAuthnRequest {
    pub id: String,
    pub raw_id: String,
    pub response: WebAuthnAuthenticatorAttestationResponse,
    pub authenticator_attachement: String,
    pub client_extension_results: WebAuthnAuthenticationExtensionsClientOutputs,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, ToSchema, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct VerifyWebAuthnResponse {}

#[utoipa::path(
    post,
    path = "login-actions/verify-webauthn",
    tag = "auth",
    summary = "Finalize a webauthn authentication",
    description = "Either create a webauthn credential or send the signature for an authentication attempt",
    responses(
        (status = 200, body = VerifyWebAuthnResponse),
    )
)]
pub async fn verify_webauthn(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<VerifyWebAuthnRequest>,
) -> Result<Response<VerifyWebAuthnResponse>, ApiError> {
    let authenticator_credential =
        WebAuthnCredentialId::decode_and_verify(payload.id, payload.raw_id)
            .map_err(|msg| ApiError::BadRequest(msg))?;

    let input = WebAuthnCredentialCreationInput {
        credential: authenticator_credential,
        response: payload.response,
        typ: payload.typ,
    };

    let _output = state
        .service
        .finalize_webauthn_credential_creation(identity, input)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(VerifyWebAuthnResponse {}))
}

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use axum::{Extension, extract::State};
use ferriskey_core::domain::trident::entities::webauthn::{
    WebAuthnAuthenticationExtensionsClientOutputs, WebAuthnAuthenticatorAttestationResponseJSON,
};
use ferriskey_core::domain::trident::ports::{TridentService, WebAuthnValidatePublicKeyInput};
use ferriskey_core::domain::{
    authentication::value_objects::Identity, trident::entities::WebAuthnCredentialIdGroup,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ValidatePublicKeyRequest {
    pub id: String,
    pub raw_id: String,
    pub response: WebAuthnAuthenticatorAttestationResponseJSON,
    pub authenticator_attachment: String,
    pub client_extension_results: WebAuthnAuthenticationExtensionsClientOutputs,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, ToSchema, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidatePublicKeyResponse {}

#[utoipa::path(
    post,
    path = "/protocol/webauthn/validate-public-key",
    tag = "auth",
    summary = "Validate and save a webauthn public key",
    description = "Saving a webauthn public key to use it for authentication attempts or MFA later.",
    request_body = ValidatePublicKeyRequest,
    responses(
        (status = 200, body = ValidatePublicKeyResponse),
    )
)]
pub async fn webauthn_validate_public_key(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ValidatePublicKeyRequest>,
) -> Result<Response<ValidatePublicKeyResponse>, ApiError> {
    let authenticator_credential =
        WebAuthnCredentialIdGroup::decode_and_verify(payload.id, payload.raw_id)
            .map_err(|msg| ApiError::BadRequest(msg))?;

    let response_object = payload
        .response
        .decode_and_verify()
        .map_err(|msg| ApiError::BadRequest(msg))?;

    let input = WebAuthnValidatePublicKeyInput {
        credential: authenticator_credential,
        response: response_object,
        typ: payload.typ,
    };

    let _output = state
        .service
        .webauthn_validate_public_key(identity, input)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ValidatePublicKeyResponse {}))
}

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::trident::entities::{
    WebAuthnAttestationConveyance, WebAuthnAttestationFormat, WebAuthnChallenge,
    WebAuthnCredentialDescriptor, WebAuthnHint, WebAuthnPubKeyCredParams, WebAuthnRelayingParty,
    WebAuthnUser,
};
use ferriskey_core::domain::trident::ports::{
    WebAuthnChallengeCreationInput, WebAuthnChallengeCreationOutput, TridentService,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChallengeWebAuthnRequest {}

/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialrpentity
/// A tad bit repetetitive but its explicit
#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeWebAuthnResponse {
    pub challenge: WebAuthnChallenge,
    pub rp: WebAuthnRelayingParty,
    pub user: WebAuthnUser,
    pub attestation: WebAuthnAttestationConveyance,
    pub attestation_formats: Vec<WebAuthnAttestationFormat>,
    pub pub_key_cred_params: Vec<WebAuthnPubKeyCredParams>,
    pub exclude_credentials: Vec<WebAuthnCredentialDescriptor>,
    pub hints: Vec<WebAuthnHint>,
    pub timeout: u64,
}

impl From<WebAuthnChallengeCreationOutput> for ChallengeWebAuthnResponse {
    fn from(output: WebAuthnChallengeCreationOutput) -> Self {
        Self {
            challenge: output.challenge,
            rp: output.rp,
            user: output.user,
            attestation: output.attestation,
            attestation_formats: output.attestation_formats,
            pub_key_cred_params: output.pub_key_cred_params,
            exclude_credentials: output.exclude_credentials,
            hints: output.hints,
            timeout: output.timeout,
        }
    }
}

#[utoipa::path(
    post,
    path = "/login-actions/challenge-webauthn",
    tag = "auth",
    summary = "Receive a WebAuthn challenge",
    description = "Provides a full PublicKeyCredentialCreationOption payload for WebAuthn credential creation/authentication. The payload contains the challenge to resolve in B64Url form as described in the specs. The content is described here: https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptions.",
    responses(
        (status = 200, body = ChallengeWebAuthnResponse)
    )
)]
pub async fn challenge_webauthn(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
) -> Result<Response<ChallengeWebAuthnResponse>, ApiError> {
    let session_code = cookie.get("FERRISKEY_SESSION").unwrap();
    let session_code = session_code.value().to_string();

    let server_host = state.args.server.host.clone();

    let output = state
        .service
        .webauthn_challenge_for_credential_creation(
            identity,
            WebAuthnChallengeCreationInput {
                session_code,
                server_host,
            },
        )
        .await
        .map_err(ApiError::from)?;

    let response = ChallengeWebAuthnResponse::from(output);
    Ok(Response::OK(response))
}

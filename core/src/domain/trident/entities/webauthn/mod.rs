mod serde;

/// Names in here are a bit verbose.
/// I tried to match as much as possible the definition of the spec:
/// https://w3c.github.io/webauthn
/// Hence the redunduncy at some places
use base64::Engine;
use base64::prelude::*;

use super::SigningAlgorithm;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::credential::entities::Credential;
use crate::domain::credential::entities::CredentialData;
use crate::domain::user::entities::User;
use ::serde::{Deserialize, Serialize};
use rand::prelude::*;
use uuid::Uuid;

#[cfg(feature = "utoipa_support")]
use utoipa::ToSchema;

pub fn spec_encode(bytes: &[u8]) -> String {
    BASE64_URL_SAFE_NO_PAD.encode(bytes)
}

pub fn spec_decode<T: Sized + From<Vec<u8>>>(b64_data: &str) -> Result<T, ()> {
    BASE64_URL_SAFE_NO_PAD
        .decode(b64_data)
        .map(T::from)
        .map_err(|_| ())
}

/// A Webauthn challenge is sent to a user both to create a webauthn credential
/// and to verify an authentication attempt with a webauthn credential
#[derive(Debug)]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub struct WebAuthnChallenge(pub Vec<u8>);

impl WebAuthnChallenge {
    /// Generates a random 16 bytes number as a challenge
    /// As specified by the spec, the challenge must be at least 16 bytes long:
    /// https://w3c.github.io/webauthn/#sctn-cryptographic-challenges
    pub fn generate() -> Result<Self, CoreError> {
        let mut bytes = [0u8; 16];
        rand::thread_rng()
            .try_fill_bytes(&mut bytes)
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(WebAuthnChallenge(bytes.to_vec()))
    }
}

impl From<Vec<u8>> for WebAuthnChallenge {
    fn from(v: Vec<u8>) -> Self {
        WebAuthnChallenge(v)
    }
}

/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialrpentity
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub struct WebAuthnRelayingParty {
    pub id: String,

    /// Deprecated in spec, recommended to be == to rp_id
    pub name: String,
}

/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialuserentityjson
/// A user representation ready to be serialized to JSON with compliant encoding for ID
/// This is why `id` is a String and not a Uuid here
#[derive(Debug)]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub struct WebAuthnUser {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
}

impl From<User> for WebAuthnUser {
    fn from(user: User) -> Self {
        WebAuthnUser {
            id: user.id,
            name: user.email,
            display_name: user.username,
        }
    }
}

/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialparameters
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub struct WebAuthnPubKeyCredParams {
    #[serde(rename = "type")]
    typ: String,
    alg: SigningAlgorithm,
}

impl WebAuthnPubKeyCredParams {
    pub fn new(alg: SigningAlgorithm) -> Self {
        WebAuthnPubKeyCredParams {
            typ: "public-key".to_string(), // Only valid value for now,
            alg,
        }
    }
}

/// https://w3c.github.io/webauthn/#enumdef-authenticatortransport
#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub enum WebAuthnAuthenticatorTransport {
    Usb,
    Ble,
    Nfc,
    SmartCard,
    Hybrid,
    Internal,
}

/// https://w3c.github.io/webauthn/#enumdef-userverificationrequirement
#[derive(Serialize, Deserialize, Debug, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WebAuthnUserVerificationRequirement {
    Required,
    Preferred,
    Discouraged,
}

/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialuserentityjson
/// Ready for JSON serialization, hence why the ID is string and not Uuid
///
/// Field description: https://w3c.github.io/webauthn/#dictdef-publickeycredentialdescriptor
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub struct WebAuthnPublicKeyCredentialDescriptor {
    #[serde(rename = "type")]
    pub typ: String,
    pub id: WebAuthnCredentialId,
    pub transports: Option<Vec<WebAuthnAuthenticatorTransport>>,
}

impl TryFrom<Credential> for WebAuthnPublicKeyCredentialDescriptor {
    type Error = CoreError;
    fn try_from(credential: Credential) -> Result<Self, Self::Error> {
        assert_eq!(
            credential.credential_type, "webauthn-public-key-credential",
            "The credential passed to WebAuthnPublicKeyCredentialDescriptor must be of type 'webauthn-public-key-credential'"
        );

        assert!(
            matches!(credential.credential_data, CredentialData::WebAuthn { .. }),
            "The credential_data type must be WebAuthn"
        );

        let CredentialData::WebAuthn { transports, .. } = credential.credential_data else {
            unreachable!()
        };

        let id = if let Some(id) = credential.webauthn_credential_id {
            id
        } else {
            return Err(CoreError::Invalid);
        };

        Ok(Self {
            typ: "public-key".to_string(), // Only valid value in spec for now
            id,
            transports: Some(transports),
        })
    }
}

/// https://w3c.github.io/webauthn/#attestation-conveyance
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub enum WebAuthnAttestationConveyance {
    None,
    Indirect,
    Direct,
    Enterprise,
}

/// 1. https://w3c.github.io/webauthn/#dom-publickeycredentialcreationoptions-attestationformats
/// 2. https://www.iana.org/assignments/webauthn/webauthn.xhtml
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub enum WebAuthnAttestationFormat {
    Packed,
    Tpm,
    AndroidKey,
    AndroidSafetynet,
    #[serde(rename = "fido-u2f")]
    FidoU2F,
    Apple,
    None,
}

/// https://w3c.github.io/webauthn/#enumdef-publickeycredentialhint
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub enum WebAuthnHint {
    SecurityKey,
    ClientDevice,
    Hybrid,
}

/// The PublicKeyCredentialCreationOptions object can be filled by the client completely.
/// To make things easier, we'll provide the complete object from the server when the user requests
/// a challenge.
/// This makes the authentication flow more streamline and has the benefit of giving the
/// server capabilities at the same time
///
/// Because encoding cannot fail, this object is automatically encoded when being serialized.
/// The encoding complies with the spec for PublicKeyCredentialCreationOptionsJson
///
/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptions
/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptionsjson
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub struct WebAuthnPublicKeyCredentialCreationOptions {
    pub challenge: WebAuthnChallenge,
    pub rp: WebAuthnRelayingParty,
    pub user: WebAuthnUser,
    pub attestation: WebAuthnAttestationConveyance,
    pub attestation_formats: Vec<WebAuthnAttestationFormat>,
    pub pub_key_cred_params: Vec<WebAuthnPubKeyCredParams>,
    pub exclude_credentials: Vec<WebAuthnPublicKeyCredentialDescriptor>,
    pub hints: Vec<WebAuthnHint>,
    pub timeout: u64,
    pub extensions: WebAuthnAuthenticationExtensionsClientInputs,
}

#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnPublicKeyCredentialRequestOptions {
    pub challenge: WebAuthnChallenge,
    pub timeout: u64,
    pub rp_id: String,
    pub allow_credentials: Vec<WebAuthnPublicKeyCredentialDescriptor>,
    pub user_verification: WebAuthnUserVerificationRequirement,
    pub hints: Vec<WebAuthnHint>,
    pub extensions: WebAuthnAuthenticationExtensionsClientInputs,
}

/// https://w3c.github.io/webauthn/#credential-id
#[derive(Clone, Debug, ToSchema, PartialEq, Eq, Ord, PartialOrd)]
pub struct WebAuthnCredentialId(pub Vec<u8>);

impl WebAuthnCredentialId {
    pub const MAX_BYTE_LEN: u32 = 1023;
}

impl From<Vec<u8>> for WebAuthnCredentialId {
    fn from(v: Vec<u8>) -> Self {
        WebAuthnCredentialId(v)
    }
}

/// This is just a non-standard data structure to manipulate raw_id and id together
pub struct WebAuthnCredentialIdGroup {
    pub id: WebAuthnCredentialId,
    pub raw_id: Vec<u8>,
}

impl WebAuthnCredentialIdGroup {
    /// This function returns a decoded and validated WebAuthnCredentialId package or an error
    /// message as a string.
    /// The error message is non-punctuated and non-capitalized.
    /// It must be tweaked or formatted and is destined to the user.
    /// This function cannot fail due to server error assuming the input data have been
    /// deserialized correctly
    ///
    ///
    /// Note for future modifications:
    /// If this function was to ever fail due to server error, the return type MUST be changed in
    /// such a way that it's easy to differentiate user-destined messages and server-destined ones
    pub fn decode_and_verify(id: String, raw_id: String) -> Result<Self, String> {
        let id = BASE64_URL_SAFE_NO_PAD
            .decode(id)
            .map_err(|_| "failed to decode id".to_string())?;

        let raw_id = BASE64_URL_SAFE_NO_PAD
            .decode(raw_id)
            .map_err(|_| "failed to decode raw_id".to_string())?;

        Ok(Self {
            id: WebAuthnCredentialId(id),
            raw_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema))]
pub struct WebAuthnAttestationObject(Vec<u8>);

impl From<Vec<u8>> for WebAuthnAttestationObject {
    fn from(v: Vec<u8>) -> Self {
        WebAuthnAttestationObject(v)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema))]
pub struct WebAuthnPublicKey(pub Vec<u8>);

impl WebAuthnPublicKey {
    /// This byte length is arbitrary and should cover most use cases
    pub const MAX_BYTE_LEN: u32 = 512;
}

impl From<Vec<u8>> for WebAuthnPublicKey {
    fn from(v: Vec<u8>) -> Self {
        WebAuthnPublicKey(v)
    }
}

/// A required empty object
/// https://w3c.github.io/webauthn/#dictdef-authenticationextensionsclientinputs
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct WebAuthnAuthenticationExtensionsClientInputs {}

/// A required empty object
/// https://w3c.github.io/webauthn/#iface-authentication-extensions-client-outputs
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema))]
pub struct WebAuthnAuthenticationExtensionsClientOutputs {}

/// This data structure contains the decoded a verified data
/// from the client, ready to be inserted into the database.
///
/// https://w3c.github.io/webauthn/#authenticatorattestationresponse
pub struct WebAuthnAuthenticatorAttestationResponse {
    pub client_data_json: String,
    pub transports: Vec<WebAuthnAuthenticatorTransport>,
    pub public_key: WebAuthnPublicKey,
    pub public_key_algorithm: SigningAlgorithm,
    pub attestation_object: WebAuthnAttestationObject,
}

/// The encoded version of the AuthenticatorAttestationResponse object
/// Meant to be sent over the wire as JSON format
/// This is a transitionary object, it must be decoded and verified before being used
/// https://w3c.github.io/webauthn/#dictdef-authenticatorattestationresponsejson
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema))]
pub struct WebAuthnAuthenticatorAttestationResponseJSON {
    #[serde(rename = "clientDataJSON")]
    pub client_data_json: String,
    pub authenticator_data: String,
    pub transports: Vec<WebAuthnAuthenticatorTransport>,
    pub public_key: String,
    pub public_key_algorithm: SigningAlgorithm,
    pub attestation_object: String,
}

impl WebAuthnAuthenticatorAttestationResponseJSON {
    /// Generate a usable AuthenticatorAttestationResponse
    /// This function verifies that the attestation response is conform with the spec
    pub fn decode_and_verify(&self) -> Result<WebAuthnAuthenticatorAttestationResponse, String> {
        let client_data_json = BASE64_URL_SAFE_NO_PAD
            .decode(self.client_data_json.clone())
            .map_err(|_| "failed to decode clientDataJSON as a Base64 URL field".to_string())?;

        let client_data_json = String::from_utf8(client_data_json.clone())
            .map_err(|_| "failed to decode clientDataJSON as a valid UTF8 string".to_string())?;

        let public_key = spec_decode(&self.public_key)
            .map_err(|_| "failed to decode publicKey as a Base64 URL field".to_string())?;

        let attestation_object = spec_decode(&self.attestation_object)
            .map_err(|_| "failed to decode attestationObject as a Base64 URL field".to_string())?;

        Ok(WebAuthnAuthenticatorAttestationResponse {
            client_data_json,
            transports: self.transports.clone(),
            public_key,
            public_key_algorithm: self.public_key_algorithm.clone(),
            attestation_object,
        })
    }
}

/// https://w3c.github.io/webauthn/#authenticatorassertionresponse
pub struct WebAuthnAuthenticatorAssertionResponse {
    pub client_data_json: String,
    pub authenticator_data: Vec<u8>,
    pub signature: Vec<u8>,
    pub user_handle: Vec<u8>,
}

// Implemented for symetry with AuthenticatorAttestationResponse
// This struct is actually not needed and a simple Deserialize impl would suffice
/// https://w3c.github.io/webauthn/#dom-authenticatorassertionresponsejson-clientdatajson
pub struct WebAuthnAuthenticatorAssertionResponseJSON {
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
    pub user_handle: String,
}

impl WebAuthnAuthenticatorAssertionResponseJSON {
    pub fn decode(&self) -> Result<WebAuthnAuthenticatorAssertionResponse, String> {
        let client_data_json = BASE64_URL_SAFE_NO_PAD
            .decode(&self.client_data_json)
            .map_err(|_| "failed to decode client_data_json as base64 url string".to_string())?;

        let client_data_json = String::from_utf8(client_data_json)
            .map_err(|_| "failed to decode client_data_json as a valid UTF8 string".to_string())?;

        let authenticator_data = BASE64_URL_SAFE_NO_PAD
            .decode(&self.authenticator_data)
            .map_err(|_| "failed to decode authenticator_data as base64 url string".to_string())?;

        let signature = BASE64_URL_SAFE_NO_PAD
            .decode(&self.signature)
            .map_err(|_| "failed to decode signature as base64 url string".to_string())?;

        let user_handle = BASE64_URL_SAFE_NO_PAD
            .decode(&self.user_handle)
            .map_err(|_| "failed to decode user_handle as base64 url string".to_string())?;
        Ok(WebAuthnAuthenticatorAssertionResponse {
            client_data_json,
            authenticator_data,
            signature,
            user_handle,
        })
    }
}

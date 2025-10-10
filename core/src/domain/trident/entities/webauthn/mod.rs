mod serde;

/// Names in here are a bit verbose.
/// I tried to match as much as possible the definition of the spec:
/// https://w3c.github.io/webauthn
/// Hence the redunduncy at some places
use base64::Engine;
use base64::prelude::*;
use x509_parser::prelude::FromDer;
use x509_parser::public_key::PublicKey;
use x509_parser::x509::SubjectPublicKeyInfo;

use super::SigningAlgorithm;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::credential::entities::Credential;
use crate::domain::credential::entities::CredentialData;
use crate::domain::user::entities::User;
use ::serde::{Deserialize, Serialize};
use p256::ecdsa::{Signature, VerifyingKey};
use rand::prelude::*;
use sha2::{Digest, Sha256};
use signature::Verifier;
use utoipa::ToSchema;
use uuid::Uuid;

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
#[derive(Debug, ToSchema, PartialEq, Eq)]
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
#[derive(Serialize, Deserialize, Debug, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnRelayingParty {
    pub id: String,

    /// Deprecated in spec, recommended to be == to rp_id
    pub name: String,
}

/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialuserentityjson
/// A user representation ready to be serialized to JSON with compliant encoding for ID
/// This is why `id` is a String and not a Uuid here
#[derive(Debug, ToSchema, PartialEq, Eq)]
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
#[derive(Serialize, Deserialize, Debug, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
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
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Ord, PartialOrd, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
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
#[derive(Serialize, Deserialize, Debug, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
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
#[derive(Serialize, Deserialize, Debug, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WebAuthnAttestationConveyance {
    None,
    Indirect,
    Direct,
    Enterprise,
}

/// 1. https://w3c.github.io/webauthn/#dom-publickeycredentialcreationoptions-attestationformats
/// 2. https://www.iana.org/assignments/webauthn/webauthn.xhtml
#[derive(Serialize, Deserialize, Debug, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
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
#[derive(Serialize, Deserialize, Debug, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
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
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
pub struct WebAuthnAttestationObject(Vec<u8>);

impl From<Vec<u8>> for WebAuthnAttestationObject {
    fn from(v: Vec<u8>) -> Self {
        WebAuthnAttestationObject(v)
    }
}

/// A public key as a DER SubjectPublicKeyInfo object
/// https://w3c.github.io/webauthn/#dom-authenticatorattestationresponse-getpublickey
/// https://datatracker.ietf.org/doc/html/rfc5280#section-4.1.2.7
#[derive(Debug, Clone, ToSchema, Eq, PartialEq, Ord, PartialOrd)]
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
#[derive(Debug, Deserialize, ToSchema, PartialEq, Eq)]
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
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
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

impl WebAuthnAuthenticatorAssertionResponse {
    pub fn verify(
        &self,
        _challenge: WebAuthnChallenge,
        pub_key: WebAuthnPublicKey,
    ) -> Result<bool, CoreError> {
        let client_data_hash = Sha256::digest(&self.client_data_json);
        let mut message =
            Vec::with_capacity(self.authenticator_data.len() + client_data_hash.len());
        message.extend_from_slice(&self.authenticator_data);
        message.extend_from_slice(&client_data_hash);

        let (_, spki) = SubjectPublicKeyInfo::from_der(&pub_key.0)
            .map_err(|_| {
                tracing::error!("A public key was failed to be parsed as a SPKI der object. The server should ensure that objects stored in the database are valid.");
                CoreError::InternalServerError
            })?;

        let spki = spki.parsed()
            .map_err(|_| {
                tracing::error!("A SPKI was failed to be parsed. The server should ensure that objects stored in the database are valid.");
                CoreError::InternalServerError
            })?;

        let spki = if let PublicKey::EC(ec) = spki {
            ec.data().to_vec()
        } else {
            // This would be a user error during credential creation
            tracing::error!(
                "A SPKI format is invalid. The server should have rejected that key during credential creation."
            );
            return Err(CoreError::InternalServerError);
        };

        if spki.len() != 65 {
            tracing::error!(
                "A SPKI doesn't have the expected length of 65 bytes. The server should have rejected that key during credential creation."
            );
            return Err(CoreError::InternalServerError);
        }

        let verifying_key =
            VerifyingKey::from_sec1_bytes(&spki).map_err(|_| CoreError::InternalServerError)?;

        let signature = Signature::from_der(&self.signature).map_err(|_| CoreError::Invalid)?;

        match verifying_key.verify(&message, &signature) {
            Ok(()) => Ok(true),
            // This could also be ISE but p256::Error is too opaque to know so in doubt we'll just
            // say it failed
            Err(_) => Ok(false),
        }
    }
}

// Implemented for symetry with AuthenticatorAttestationResponse
// This struct is actually not needed and a simple Deserialize impl would suffice
/// https://w3c.github.io/webauthn/#dom-authenticatorassertionresponsejson-clientdatajson
#[derive(Debug, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnAuthenticatorAssertionResponseJSON {
    #[serde(rename = "clientDataJSON")]
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

#[cfg(test)]
mod tests {
    use p256::ecdsa::{DerSignature, VerifyingKey};
    use signature::Verifier;
    use x509_parser::asn1_rs::{Length, Tag};
    use x509_parser::prelude::*;
    use x509_parser::public_key::*;

    const KEY_HEX: &'static str = "3059301306072a8648ce3d020106082a8648ce3d030107034200049a01dfcbf76919678e7649e74205769991fd393b0960d0c4728154327fb1c1b61fd6100435099ac18697e57bcb1cd54b7dec5395e4ffcb255c072bcd94cb9c3d";

    #[test]
    /// This one is not a logic test per se, rather a test to showcase how key parsing should work
    /// and the expected formats.
    fn test_der_parser() {
        // This is a public key that was generated during developement.
        // It's the direct publicKey object generated by a client's authenticator without any
        // treatement. It's expected to be a DER X.509 SubjectPublicKeyInfo object
        // ASN.1:
        /*
        SubjectPublicKeyInfo ::= SEQUENCE {
            algorithm SEQUENCE {
                algorithm OBJECT IDENTIFIER ecPublicKey (1 2 840 10045 2 1),
                parameters OBJECT IDENTIFIER prime256v1 (1 2 840 10045 3 1 7)
            },
            subjectPublicKey BIT STRING
            'blabla bytes'H
        }
        */

        let key_bytes = hex::decode(KEY_HEX).expect("Failed to decode key as hex string");
        let (_, spki) = SubjectPublicKeyInfo::from_der(&key_bytes)
            .expect("Failed to parse key byte string as a valid der SPKI sequence");

        // algorithm field of algorithm
        let spki_algorithm_id = spki.algorithm.algorithm.to_id_string();
        assert_eq!(
            &spki_algorithm_id, "1.2.840.10045.2.1",
            "The algorithm id doesn't match"
        );

        // parameters field of algorithm
        let spki_algorithm_params = spki.algorithm.parameters.clone();
        assert!(
            matches!(spki_algorithm_params, Some(_)),
            "Expected params on the spki"
        );
        let spki_algorithm_params = spki_algorithm_params.unwrap();
        assert_eq!(spki_algorithm_params.header.tag(), Tag::Oid);
        assert_eq!(spki_algorithm_params.header.length(), Length::Definite(8));
        let spki_algorithm_params = spki_algorithm_params
            .oid()
            .expect("Failed to convert param to Oid");
        assert_eq!(
            spki_algorithm_params.to_id_string(),
            "1.2.840.10045.3.1.7",
            "The algorithm parameters id string doesn't match"
        );

        // key data
        assert_eq!(
            spki.subject_public_key.unused_bits, 0,
            "Expected 0 unused bits"
        );
        assert_eq!(
            spki.subject_public_key.data.as_ref(),
            [
                0x04, 0x9a, 0x01, 0xdf, 0xcb, 0xf7, 0x69, 0x19, 0x67, 0x8e, 0x76, 0x49, 0xe7, 0x42,
                0x05, 0x76, 0x99, 0x91, 0xfd, 0x39, 0x3b, 0x09, 0x60, 0xd0, 0xc4, 0x72, 0x81, 0x54,
                0x32, 0x7f, 0xb1, 0xc1, 0xb6, 0x1f, 0xd6, 0x10, 0x04, 0x35, 0x09, 0x9a, 0xc1, 0x86,
                0x97, 0xe5, 0x7b, 0xcb, 0x1c, 0xd5, 0x4b, 0x7d, 0xec, 0x53, 0x95, 0xe4, 0xff, 0xcb,
                0x25, 0x5c, 0x07, 0x2b, 0xcd, 0x94, 0xcb, 0x9c, 0x3d,
            ],
            "Parsed data doesn't match expected data"
        );

        // Reinterpret data, not that its needed
        let pk = spki.parsed().expect("Parsing was expected to succeed");
        assert!(
            matches!(pk, PublicKey::EC(_)),
            "Expected EC public key type"
        );
    }
}

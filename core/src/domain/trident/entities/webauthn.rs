use base64::Engine;
use base64::prelude::*;

use super::SigningAlgorithm;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::user::entities::User;
use rand::prelude::*;
use serde::de::Deserializer;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

#[cfg(feature = "utoipa_support")]
use utoipa::ToSchema;

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

    /// Straight B64 encoding of the challenge
    /// https://w3c.github.io/webauthn/#sctn-parseCreationOptionsFromJSON
    pub fn encode(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(self.0.clone())
    }
}

impl Serialize for WebAuthnChallenge {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.encode().as_str())
    }
}

impl<'de> Deserialize<'de> for WebAuthnChallenge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        WebAuthnChallenge::try_from(s)
            .map_err(|_| serde::de::Error::custom("failed to decode string as a challenge"))
    }
}

impl TryFrom<String> for WebAuthnChallenge {
    type Error = CoreError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let result = BASE64_URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|_| CoreError::Invalid)?;

        Ok(WebAuthnChallenge(result))
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

impl Serialize for WebAuthnUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut uuid = uuid::Uuid::encode_buffer();
        let uuid = self.id.hyphenated().encode_lower(&mut uuid);
        let uuid = BASE64_URL_SAFE_NO_PAD.encode(uuid);

        let mut user = serializer.serialize_struct("WebAuthnUser", 3)?;
        user.serialize_field("id", &uuid)?;
        user.serialize_field("name", &self.name)?;
        user.serialize_field("displayName", &self.display_name)?;
        user.end()
    }
}

impl<'de> Deserialize<'de> for WebAuthnUser {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct InputPayload {
            id: String,
            name: String,
            display_name: String,
        }

        let payload: InputPayload = InputPayload::deserialize(deserializer)?;

        let uuid = BASE64_URL_SAFE_NO_PAD.decode(payload.id).map_err(|_| {
            serde::de::Error::custom("failed to decode id as B64Url without padding")
        })?;
        let uuid = Uuid::from_slice(&uuid)
            .map_err(|_| serde::de::Error::custom("failed to parse id as a valid Uuid"))?;

        Ok(WebAuthnUser {
            id: uuid,
            name: payload.name,
            display_name: payload.display_name,
        })
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

#[derive(Serialize, Deserialize, Debug)]
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

/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialuserentityjson
/// Ready for JSON serialization, hence why the ID is string and not Uuid
///
/// Field description: https://w3c.github.io/webauthn/#dictdef-publickeycredentialdescriptor
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema, PartialEq, Eq))]
pub struct WebAuthnCredentialDescriptor {
    #[serde(rename = "type")]
    pub typ: String,
    pub id: String,
    pub transports: Option<Vec<WebAuthnAuthenticatorTransport>>,
}

impl WebAuthnCredentialDescriptor {
    pub fn new() -> Self {
        Self {
            typ: "public-key".to_string(), // Only valid value in spec for now
            // Must match the credential record ID, maybe we can extract it
            // from a Credential ?
            id: "0".to_string(),
            transports: None,
        }
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

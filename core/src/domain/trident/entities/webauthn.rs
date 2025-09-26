/// Names in here are a bit verbose.
/// I tried to match as much as possible the definition of the spec:
/// https://w3c.github.io/webauthn
/// Hence the redunduncy at some places
use base64::Engine;
use base64::prelude::*;

use super::SigningAlgorithm;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::user::entities::User;
use rand::prelude::*;
use serde::de::{Deserializer, Error, Unexpected};
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
    pub exclude_credentials: Vec<WebAuthnCredentialDescriptor>,
    pub hints: Vec<WebAuthnHint>,
    pub timeout: u64,
}

/// RegistrationResponse defined in the spec doesn't split credential ID
/// fields in their own structure.
/// We group them internally because it makes sense to decode and manipulate them together
/// Hence why there is no "XJson" variation of this struct like for AuthenticatorAttestationResponse
///
/// https://w3c.github.io/webauthn/#publickeycredential and https://w3c.github.io/webauthn/#dictdef-registrationresponsejson
pub struct WebAuthnCredentialId {
    pub id: String,
    pub raw_id: Vec<u8>,
}

impl WebAuthnCredentialId {
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
        let id = String::from_utf8(id).map_err(|_| "id is not a valid utf8 string")?;

        let raw_id = BASE64_URL_SAFE_NO_PAD
            .decode(raw_id)
            .map_err(|_| "failed to decode raw_id".to_string())?;

        Ok(Self { id, raw_id })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "utoipa_support", derive(ToSchema))]
pub struct WebAuthnAttestationObject(Vec<u8>);

impl WebAuthnAttestationObject {
    pub fn encode(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(&self.0)
    }

    pub fn decode(b64_data: &str) -> Result<Self, ()> {
        BASE64_URL_SAFE_NO_PAD
            .decode(b64_data)
            .map(|bytes| WebAuthnAttestationObject(bytes))
            .map_err(|_| ())
    }
}

impl Serialize for WebAuthnAttestationObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.encode().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WebAuthnAttestationObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        WebAuthnAttestationObject::decode(&value).map_err(|_| {
            D::Error::invalid_value(
                Unexpected::Str(&value),
                &"failed to decode string with B64 URL",
            )
        })
    }
}

/// This data structure contains the decoded a verified data
/// from the client, ready to be inserted into the database.
///
/// https://w3c.github.io/webauthn/#authenticatorattestationresponse
pub struct WebAuthnAuthenticatorAttestationResponse {
    pub client_data_json: String,
    pub transports: Vec<WebAuthnAuthenticatorTransport>,
    pub public_key: Vec<u8>,
    pub public_key_algorithm: SigningAlgorithm,
    pub attestation_object: WebAuthnAttestationObject,
}

impl<'de> Deserialize<'de> for WebAuthnAuthenticatorAttestationResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        /// https://w3c.github.io/webauthn/#dictdef-authenticatorattestationresponsejson
        pub struct Helper {
            #[serde(rename = "clientDataJSON")]
            pub client_data_json: String,
            pub transports: Vec<WebAuthnAuthenticatorTransport>,
            pub public_key: String,
            pub public_key_algorithm: SigningAlgorithm,
            pub attestation_object: String,
        }

        let helper = Helper::deserialize(deserializer)?;
        let client_data_json = BASE64_URL_SAFE_NO_PAD
            .decode(helper.client_data_json.clone())
            .map_err(|_| {
                D::Error::invalid_value(
                    Unexpected::Str(&helper.client_data_json),
                    &"a Base64URL encoded JSON payload",
                )
            })?;

        let client_data_json = String::from_utf8(client_data_json.clone()).map_err(|_| {
            D::Error::invalid_value(
                Unexpected::Bytes(&client_data_json),
                &"a valid UTF8 JSON payload",
            )
        })?;

        let public_key = BASE64_URL_SAFE_NO_PAD
            .decode(helper.public_key.clone())
            .map_err(|_| {
                D::Error::invalid_value(
                    Unexpected::Str(&helper.public_key),
                    &"a Base64URL encoded byte sequence",
                )
            })?;

        let attestation_object = WebAuthnAttestationObject::decode(&helper.attestation_object)
            .map_err(|_| {
                D::Error::invalid_value(
                    Unexpected::Str(&helper.attestation_object),
                    &"a Base64URL encoded byte sequence",
                )
            })?;

        Ok(Self {
            client_data_json,
            transports: helper.transports,
            public_key,
            public_key_algorithm: helper.public_key_algorithm,
            attestation_object,
        })
    }
}

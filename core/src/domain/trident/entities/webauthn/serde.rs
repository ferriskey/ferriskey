use ::serde::de::{Deserializer, Error, Unexpected};
use ::serde::ser::SerializeStruct;
/// Contains custom impl of Serialize or Deserialize for various types
use ::serde::{Deserialize, Serialize, Serializer};

use super::*;

impl Serialize for WebAuthnChallenge {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        spec_encode(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WebAuthnChallenge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        spec_decode(&s).map_err(|_| Error::custom("failed to decode string as a challenge"))
    }
}

impl Serialize for WebAuthnUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut uuid = Uuid::encode_buffer();
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

        let uuid = BASE64_URL_SAFE_NO_PAD
            .decode(payload.id)
            .map_err(|_| Error::custom("failed to decode id as B64Url without padding"))?;
        let uuid = Uuid::from_slice(&uuid)
            .map_err(|_| Error::custom("failed to parse id as a valid Uuid"))?;

        Ok(WebAuthnUser {
            id: uuid,
            name: payload.name,
            display_name: payload.display_name,
        })
    }
}

impl Serialize for WebAuthnAttestationObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        spec_encode(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WebAuthnAttestationObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        spec_decode(&value).map_err(|_| {
            D::Error::invalid_value(
                Unexpected::Str(&value),
                &"failed to decode string with B64 URL",
            )
        })
    }
}

impl Serialize for WebAuthnPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        spec_encode(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WebAuthnPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        spec_decode(&value).map_err(|_| {
            D::Error::invalid_value(
                Unexpected::Str(&value),
                &"failed to decode string with B64 URL",
            )
        })
    }
}

impl Serialize for WebAuthnCredentialId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        spec_encode(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WebAuthnCredentialId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        spec_decode(&value).map_err(|_| {
            D::Error::invalid_value(
                Unexpected::Str(&value),
                &"failed to decode string with B64 URL",
            )
        })
    }
}

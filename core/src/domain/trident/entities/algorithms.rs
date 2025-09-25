use serde::de::{Deserialize, Deserializer, Error, Unexpected};
use serde::{Serialize, Serializer};

/// Enum representation of various signing algorithms.
/// Can be converted to an i16 representing their COSE identifier
///
/// If utoipa_support feature is enabled, the generated schema will incorrectly use enum string
/// values instead of the COSE identifiers.
/// TODO: Manually implement ToSchema
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(i16)]
#[cfg_attr(feature = "utoipa_support", derive(utoipa::ToSchema), schema(examples(-7, -257)))]
pub enum SigningAlgorithm {
    ES256 = -7,
    RS256 = -257,
    EdDSA = -8,
}

impl SigningAlgorithm {
    pub fn cose_identifier(&self) -> i16 {
        *self as i16
    }
}

impl TryFrom<i16> for SigningAlgorithm {
    type Error = ();
    fn try_from(v: i16) -> Result<Self, Self::Error> {
        match v {
            -7 => Ok(SigningAlgorithm::ES256),
            -9 => Ok(SigningAlgorithm::EdDSA),
            -257 => Ok(SigningAlgorithm::RS256),
            _ => Err(()),
        }
    }
}

impl Serialize for SigningAlgorithm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i16(self.cose_identifier())
    }
}

impl<'de> Deserialize<'de> for SigningAlgorithm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i16::deserialize(deserializer)?;
        match value.try_into() {
            Ok(alg) => Ok(alg),
            Err(_) => Err(D::Error::invalid_value(
                Unexpected::Signed(value as i64),
                &"a valid COSE signing algorithm identifier",
            )),
        }
    }
}

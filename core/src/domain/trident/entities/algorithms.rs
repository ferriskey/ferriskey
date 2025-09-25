use serde::{Deserialize, Serialize};

/// Enum representation of various signing algorithms.
/// Can be converted to an i16 representing their COSE identifier
///
/// If utoipa_support feature is enabled, the generated schema will incorrectly use enum string
/// values instead of the COSE identifiers.
/// TODO: Manually implement ToSchema
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
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

use serde::Serializer;

pub enum SigningAlgorithm {
    ES256,
    RS256,
    EdDSA,
}

impl SigningAlgorithm {
    pub fn cose_identifier(&self) -> i16 {
        match self {
            SigningAlgorithm::ES256 => -7,
            SigningAlgorithm::RS256 => -257,
            SigningAlgorithm::EdDSA => -8,
        }
    }
}

pub fn serialize_signing_algorithm<S>(
    value: &SigningAlgorithm,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i16(value.cose_identifier())
}

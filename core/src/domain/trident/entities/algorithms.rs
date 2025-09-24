use serde::{Serializer, Deserializer};
use serde::de::{Visitor, Error};
use std::fmt;

#[derive(Debug)]
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

pub fn deserialize_signing_algorithm<'de, D>(
    deserializer: D,
) -> Result<SigningAlgorithm, D::Error>
where
    D: Deserializer<'de>,
{
    struct AlgoVisitor;

    impl<'de> Visitor<'de> for AlgoVisitor {
        type Value = SigningAlgorithm;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a negative i16 representing a COSE algorithm id")
        }

        fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v {
                -7 => Ok(SigningAlgorithm::ES256),
                -257 => Ok(SigningAlgorithm::RS256),
                -8 => Ok(SigningAlgorithm::EdDSA),
                _ => Err(E::custom(format!("unknown COSE algorithm id: {}", v))),
            }
        }

        // serde might also pass numbers as i32/u32/etc, so cover those:
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if v < i16::MIN as i64 || v > i16::MAX as i64 {
                return Err(E::custom(format!("out of range i16: {}", v)));
            }
            self.visit_i16(v as i16)
        }
    }

    deserializer.deserialize_i16(AlgoVisitor)
}

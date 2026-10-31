use std::fmt;

use aws_lc_rs::{
    digest::{SHA256, digest},
    rand,
};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use thiserror::Error;

const PREFIX: &str = "fkr_";
const SECRET_BYTES: usize = 32;
const ENCODED_LEN: usize = 43;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("malformed opaque refresh token")]
pub struct MalformedOpaqueRefreshToken;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RefreshTokenHash(pub [u8; 32]);

impl fmt::Debug for RefreshTokenHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RefreshTokenHash(..)")
    }
}

impl RefreshTokenHash {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for RefreshTokenHash {
    type Error = MalformedOpaqueRefreshToken;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        <[u8; 32]>::try_from(bytes)
            .map(Self)
            .map_err(|_| MalformedOpaqueRefreshToken)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct OpaqueRefreshToken(String);

impl fmt::Debug for OpaqueRefreshToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("fkr_…")
    }
}

impl OpaqueRefreshToken {
    #[must_use]
    pub fn generate() -> Self {
        let mut secret = [0u8; SECRET_BYTES];
        rand::fill(&mut secret).expect("the operating system random generator is unavailable");
        Self(format!("{PREFIX}{}", BASE64_URL_SAFE_NO_PAD.encode(secret)))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn hash(&self) -> RefreshTokenHash {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(digest(&SHA256, self.0.as_bytes()).as_ref());
        RefreshTokenHash(hash)
    }
}

impl TryFrom<&str> for OpaqueRefreshToken {
    type Error = MalformedOpaqueRefreshToken;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let encoded = value
            .strip_prefix(PREFIX)
            .filter(|encoded| encoded.len() == ENCODED_LEN)
            .ok_or(MalformedOpaqueRefreshToken)?;
        let decoded = BASE64_URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| MalformedOpaqueRefreshToken)?;
        if decoded.len() != SECRET_BYTES {
            return Err(MalformedOpaqueRefreshToken);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PresentedRefreshToken {
    Opaque(OpaqueRefreshToken),
    Legacy(String),
}

impl PresentedRefreshToken {
    pub fn parse(token: &str) -> Result<Self, MalformedOpaqueRefreshToken> {
        if token.contains('.') {
            return Ok(Self::Legacy(token.to_owned()));
        }
        OpaqueRefreshToken::try_from(token).map(Self::Opaque)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_token_has_the_documented_shape() {
        let token = OpaqueRefreshToken::generate();
        let text = token.as_str();

        assert!(text.starts_with("fkr_"), "{text}");
        assert_eq!(text.len(), 47, "{text}");
        assert!(
            text[4..]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "{text}"
        );
    }

    #[test]
    fn a_generated_token_survives_a_parse_round_trip() {
        let token = OpaqueRefreshToken::generate();

        let parsed = OpaqueRefreshToken::try_from(token.as_str()).expect("a generated token");

        assert_eq!(parsed, token);
        assert_eq!(parsed.hash(), token.hash());
    }

    #[test]
    fn two_generated_tokens_differ() {
        assert_ne!(
            OpaqueRefreshToken::generate(),
            OpaqueRefreshToken::generate()
        );
    }

    #[test]
    fn the_hash_is_the_sha256_of_the_token_text() {
        let token = OpaqueRefreshToken::try_from("fkr_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
            .expect("a well-formed token");

        let expected = digest(&SHA256, b"fkr_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");

        assert_eq!(token.hash().0.as_slice(), expected.as_ref());
    }

    #[test]
    fn debug_never_prints_the_secret() {
        let token = OpaqueRefreshToken::generate();

        let rendered = format!("{token:?}");

        assert_eq!(rendered, "fkr_…");
        assert!(!rendered.contains(&token.as_str()[4..]));
    }

    #[test]
    fn malformed_tokens_are_rejected() {
        let valid = OpaqueRefreshToken::generate();
        let text = valid.as_str();
        let cases = [
            ("empty", String::new()),
            ("prefix only", "fkr_".to_string()),
            ("wrong prefix", text.replacen("fkr_", "fks_", 1)),
            ("too short", text[..text.len() - 1].to_string()),
            ("too long", format!("{text}A")),
            (
                "outside the alphabet",
                format!("{}+", &text[..text.len() - 1]),
            ),
            ("padding", format!("{}=", &text[..text.len() - 1])),
            (
                "non canonical tail",
                format!("{}B", &text[..text.len() - 1]),
            ),
        ];

        for (name, input) in cases {
            assert!(
                OpaqueRefreshToken::try_from(input.as_str()).is_err(),
                "{name}: {input}"
            );
        }
    }

    #[test]
    fn a_token_with_dots_is_presented_as_legacy_and_one_without_as_opaque() {
        let opaque = OpaqueRefreshToken::generate();

        assert_eq!(
            PresentedRefreshToken::parse("a.b.c"),
            Ok(PresentedRefreshToken::Legacy("a.b.c".to_string()))
        );
        assert_eq!(
            PresentedRefreshToken::parse(opaque.as_str()),
            Ok(PresentedRefreshToken::Opaque(opaque))
        );
        assert!(PresentedRefreshToken::parse("fkr_short").is_err());
    }

    #[test]
    fn a_hash_round_trips_through_bytes() {
        let hash = OpaqueRefreshToken::generate().hash();

        assert_eq!(RefreshTokenHash::try_from(hash.as_bytes()), Ok(hash));
        assert!(RefreshTokenHash::try_from([0u8; 5].as_slice()).is_err());
    }
}

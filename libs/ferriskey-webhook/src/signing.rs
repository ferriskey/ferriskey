//! HMAC-SHA256 signing for outbound webhook deliveries and the per-webhook secrets that back it.

use hex::encode;
use hmac::{Hmac, Mac};
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::Sha256;

/// Carries the hex-encoded output of [`sign`].
pub const SIGNATURE_HEADER: &str = "x-ferriskey-signature";

/// Carries the decimal Unix timestamp (seconds) that was folded into the signed material by
/// [`sign`]; a receiver needs this value again to reproduce the signature.
pub const TIMESTAMP_HEADER: &str = "x-ferriskey-timestamp";

/// Carries an identifier unique to this delivery attempt, for receiver-side deduplication.
pub const DELIVERY_HEADER: &str = "x-ferriskey-delivery-id";

type HmacSha256 = Hmac<Sha256>;

/// Generates a webhook secret from the OS CSPRNG: 32 bytes of randomness, hex-encoded. Not a
/// UUID: a v4 UUID commits 6 of its 128 bits to a fixed version/variant pattern and exists to be
/// unique, not to resist guessing, whereas a signing secret's entire value must resist guessing.
pub fn generate_secret() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    encode(bytes)
}

/// Signs the byte string `{timestamp}.{body}` — the decimal Unix timestamp in seconds, a literal
/// `.` (0x2E), then the raw request body — with HMAC-SHA256 under `secret`, returning the
/// hex-encoded digest. A receiver reproduces this by concatenating the value of the
/// [`TIMESTAMP_HEADER`] it received, a `.`, and the exact bytes of the body it received, then
/// comparing HMAC-SHA256 output against the [`SIGNATURE_HEADER`] value. The timestamp is folded
/// into the signed material rather than merely accompanying it: if the signature covered only
/// the body, a signature captured off the wire could be replayed indefinitely under a fresh
/// timestamp header, since nothing would tie the two together.
pub fn sign(secret: &str, timestamp: i64, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC-SHA256 accepts a signing key of any length, per the hmac crate's own documented invariant");
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(body);
    encode(mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_is_stable_for_fixed_inputs() {
        let first = sign("a-secret", 1_700_000_000, b"{\"event\":\"user.created\"}");
        let second = sign("a-secret", 1_700_000_000, b"{\"event\":\"user.created\"}");

        assert_eq!(first, second);
    }

    #[test]
    fn sign_changes_when_body_changes() {
        let original = sign("a-secret", 1_700_000_000, b"{\"event\":\"user.created\"}");
        let tampered = sign("a-secret", 1_700_000_000, b"{\"event\":\"user.deleted\"}");

        assert_ne!(original, tampered);
    }

    #[test]
    fn sign_changes_when_timestamp_changes() {
        let original = sign("a-secret", 1_700_000_000, b"{\"event\":\"user.created\"}");
        let replayed = sign("a-secret", 1_700_000_999, b"{\"event\":\"user.created\"}");

        assert_ne!(original, replayed);
    }

    #[test]
    fn sign_does_not_match_under_a_different_secret() {
        let signed_with_a = sign("secret-a", 1_700_000_000, b"{\"event\":\"user.created\"}");
        let signed_with_b = sign("secret-b", 1_700_000_000, b"{\"event\":\"user.created\"}");

        assert_ne!(signed_with_a, signed_with_b);
    }

    #[test]
    fn sign_produces_hex_encoded_sha256_length_output() {
        let signature = sign("a-secret", 1_700_000_000, b"body");

        assert_eq!(signature.len(), 64);
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_secret_produces_distinct_values() {
        let first = generate_secret();
        let second = generate_secret();

        assert_ne!(first, second);
    }

    #[test]
    fn generate_secret_is_not_a_uuid() {
        let secret = generate_secret();

        assert_eq!(secret.len(), 64);
        assert!(!secret.contains('-'));
    }
}

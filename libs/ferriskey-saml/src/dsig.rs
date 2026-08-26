use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::c14n::{C14nError, canonicalize_exclusive};

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error(transparent)]
    Canonicalisation(#[from] C14nError),
}

pub fn digest_of(xml: &str) -> Result<String, SignatureError> {
    let canonical = canonicalize_exclusive(xml)?;
    Ok(BASE64.encode(Sha256::digest(canonical.as_bytes())))
}

#[cfg(test)]
mod tests {
    use super::digest_of;

    #[test]
    fn digest_is_the_base64_sha256_of_the_canonical_form() {
        assert_eq!(
            digest_of("<a>t</a>").unwrap(),
            "n657antSFi6O3meIa2JZ1/Wp7aoBQJ1j9ixVLDSLL4w="
        );
    }

    #[test]
    fn digest_covers_the_canonical_form_not_the_source_bytes() {
        assert_eq!(
            digest_of("<a><b   >x</b></a>").unwrap(),
            digest_of("<a><b>x</b></a>").unwrap()
        );
    }
}

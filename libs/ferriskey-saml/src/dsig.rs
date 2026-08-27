use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rsa::RsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{SignatureEncoding, Signer};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::c14n::{C14nError, canonicalize_exclusive};

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error(transparent)]
    Canonicalisation(#[from] C14nError),

    #[error("unusable private key: {0}")]
    UnusableKey(String),

    #[error("signing failed: {0}")]
    SigningFailed(String),
}

pub const DSIG_NAMESPACE: &str = "http://www.w3.org/2000/09/xmldsig#";
pub const EXCLUSIVE_C14N: &str = "http://www.w3.org/2001/10/xml-exc-c14n#";
pub const ENVELOPED_SIGNATURE: &str = "http://www.w3.org/2000/09/xmldsig#enveloped-signature";
pub const RSA_SHA256: &str = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256";
pub const SHA256: &str = "http://www.w3.org/2001/04/xmlenc#sha256";

pub fn signed_info(reference_id: &str, digest: &str) -> String {
    let mut info = String::new();
    info.push_str("<ds:SignedInfo xmlns:ds=\"");
    info.push_str(DSIG_NAMESPACE);
    info.push_str("\"><ds:CanonicalizationMethod Algorithm=\"");
    info.push_str(EXCLUSIVE_C14N);
    info.push_str("\"></ds:CanonicalizationMethod><ds:SignatureMethod Algorithm=\"");
    info.push_str(RSA_SHA256);
    info.push_str("\"></ds:SignatureMethod><ds:Reference URI=\"#");
    info.push_str(reference_id);
    info.push_str("\"><ds:Transforms><ds:Transform Algorithm=\"");
    info.push_str(ENVELOPED_SIGNATURE);
    info.push_str("\"></ds:Transform><ds:Transform Algorithm=\"");
    info.push_str(EXCLUSIVE_C14N);
    info.push_str("\"></ds:Transform></ds:Transforms><ds:DigestMethod Algorithm=\"");
    info.push_str(SHA256);
    info.push_str("\"></ds:DigestMethod><ds:DigestValue>");
    info.push_str(digest);
    info.push_str("</ds:DigestValue></ds:Reference></ds:SignedInfo>");
    info
}

pub fn rsa_sha256_signature(private_key_pem: &str, data: &[u8]) -> Result<String, SignatureError> {
    let key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)
        .map_err(|e| SignatureError::UnusableKey(e.to_string()))?;
    let signature = SigningKey::<Sha256>::new(key)
        .try_sign(data)
        .map_err(|e| SignatureError::SigningFailed(e.to_string()))?;

    Ok(BASE64.encode(signature.to_bytes()))
}

pub fn digest_of(xml: &str) -> Result<String, SignatureError> {
    let canonical = canonicalize_exclusive(xml)?;
    Ok(BASE64.encode(Sha256::digest(canonical.as_bytes())))
}

#[cfg(test)]
mod tests {
    use super::{digest_of, rsa_sha256_signature, signed_info};

    const SIGNING_KEY: &str = include_str!("../tests/fixtures/signing-key.pem");

    #[test]
    fn digest_is_the_base64_sha256_of_the_canonical_form() {
        assert_eq!(
            digest_of("<a>t</a>").expect("digest"),
            "n657antSFi6O3meIa2JZ1/Wp7aoBQJ1j9ixVLDSLL4w="
        );
    }

    #[test]
    fn digest_covers_the_canonical_form_not_the_source_bytes() {
        assert_eq!(
            digest_of("<a><b   >x</b></a>").expect("digest"),
            digest_of("<a><b>x</b></a>").expect("digest")
        );
    }

    #[test]
    fn a_signature_matches_the_one_openssl_produces_for_the_same_bytes() {
        assert_eq!(
            rsa_sha256_signature(SIGNING_KEY, b"<ds:SignedInfo>x</ds:SignedInfo>").expect("sign"),
            "puCGMlB9rzu9kKVzBFIWbGd2Ap22iX1Rc2DffmrdiZ+FOoTwTMtVWsR1elIfU0aq9Meets/VaOcESYoR5EitRG5lhtBYzcBPE9p+fki+PTSPZqQCqPQZYGkzuWx1kDa6/OqLjVaa3KgMR1534nWTkRrtYhla2H+uc3bEKJnWJu3/E68fNa9mT1XbEl3r4PDcdpJUnXYw6hcSphnCtJvAh3PVhOdwW8P92/yvMAxNjNU/kNrFws51Ofr2bG8bz+RAIRXb1f3z2/3STtk4IQHxRapW5GYnOLOd01QsuXPPzpLOjXl5tyNIodt5En8VA5tUi6ffC6iw353HUyZ6RY1CIg=="
        );
    }

    #[test]
    fn a_malformed_private_key_is_refused() {
        assert!(rsa_sha256_signature("not a pem", b"x").is_err());
    }

    #[test]
    fn signed_info_declares_the_namespace_it_inherits_so_it_canonicalises_standalone() {
        let info = signed_info("_abc", "DIGEST");
        assert!(
            info.starts_with(r##"<ds:SignedInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#">"##)
        );
        assert_eq!(
            crate::c14n::canonicalize_exclusive(&info).expect("canonicalise"),
            info
        );
    }

    #[test]
    fn signed_info_references_the_element_and_carries_its_digest() {
        let info = signed_info("_abc", "DIGEST");
        assert!(info.contains(r##"<ds:Reference URI="#_abc">"##));
        assert!(info.contains("<ds:DigestValue>DIGEST</ds:DigestValue>"));
        assert!(
            info.contains(r##"Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature""##)
        );
        assert!(
            info.contains(r##"Algorithm="http://www.w3.org/2001/04/xmldsig-more#rsa-sha256""##)
        );
    }
}

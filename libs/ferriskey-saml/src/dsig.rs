use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rsa::RsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{SignatureEncoding, Signer};
use sha2::{Digest, Sha256};
use thiserror::Error;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::c14n::{C14nError, canonicalize_element, canonicalize_exclusive};

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error(transparent)]
    Canonicalisation(#[from] C14nError),

    #[error("unusable private key: {0}")]
    UnusableKey(String),

    #[error("signing failed: {0}")]
    SigningFailed(String),

    #[error("element {0} already carries a signature")]
    AlreadySigned(String),

    #[error("element {0} is empty and cannot hold a signature")]
    EmptyElement(String),

    #[error("no element carries ID={0}")]
    UnknownElement(String),
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

pub fn sign_enveloped(
    xml: &str,
    element_id: &str,
    private_key_pem: &str,
    certificate_base64_der: &str,
) -> Result<String, SignatureError> {
    let insertion = signature_insertion_point(xml, element_id)?;
    let digest = digest_of_element(xml, element_id)?;
    let info = signed_info(element_id, &digest);
    let signature_value = rsa_sha256_signature(private_key_pem, info.as_bytes())?;

    let mut block = String::from("<ds:Signature xmlns:ds=\"");
    block.push_str(DSIG_NAMESPACE);
    block.push_str("\">");
    block.push_str(&info.replace(&format!(" xmlns:ds=\"{DSIG_NAMESPACE}\""), ""));
    block.push_str("<ds:SignatureValue>");
    block.push_str(&signature_value);
    block.push_str("</ds:SignatureValue><ds:KeyInfo><ds:X509Data><ds:X509Certificate>");
    block.push_str(certificate_base64_der);
    block.push_str("</ds:X509Certificate></ds:X509Data></ds:KeyInfo></ds:Signature>");

    let mut signed = String::with_capacity(xml.len() + block.len());
    signed.push_str(&xml[..insertion]);
    signed.push_str(&block);
    signed.push_str(&xml[insertion..]);
    Ok(signed)
}

fn digest_of_element(xml: &str, element_id: &str) -> Result<String, SignatureError> {
    let canonical = canonicalize_element(xml, element_id)?;
    Ok(BASE64.encode(Sha256::digest(canonical.as_bytes())))
}

fn signature_insertion_point(xml: &str, element_id: &str) -> Result<usize, SignatureError> {
    let mut reader = Reader::from_str(xml);
    let mut depth = 0usize;
    let mut target_depth: Option<usize> = None;
    let mut insertion: Option<usize> = None;

    loop {
        let event = reader.read_event().map_err(|e| {
            SignatureError::Canonicalisation(C14nError::MalformedXml(e.to_string()))
        })?;

        match event {
            Event::Start(tag) => {
                if target_depth.is_none() && carries_id(&tag, element_id)? {
                    target_depth = Some(depth);
                    insertion = Some(reader.buffer_position() as usize);
                } else if target_depth.is_some()
                    && local_name_of(tag.name().as_ref()) == b"Signature"
                {
                    return Err(SignatureError::AlreadySigned(element_id.to_owned()));
                }
                depth += 1;
            }
            Event::Empty(tag) => {
                if target_depth.is_none() && carries_id(&tag, element_id)? {
                    return Err(SignatureError::EmptyElement(element_id.to_owned()));
                }
            }
            Event::End(tag) => {
                depth = depth.saturating_sub(1);
                if target_depth == Some(depth) {
                    break;
                }
                if target_depth.is_some()
                    && depth == target_depth.unwrap_or_default() + 1
                    && local_name_of(tag.name().as_ref()) == b"Issuer"
                {
                    insertion = Some(reader.buffer_position() as usize);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    insertion.ok_or_else(|| SignatureError::UnknownElement(element_id.to_owned()))
}

fn carries_id(tag: &BytesStart<'_>, element_id: &str) -> Result<bool, SignatureError> {
    for attribute in tag.attributes() {
        let attribute = attribute.map_err(|e| {
            SignatureError::Canonicalisation(C14nError::MalformedXml(e.to_string()))
        })?;
        if attribute.key.as_ref() == b"ID" && attribute.value.as_ref() == element_id.as_bytes() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn local_name_of(qualified: &[u8]) -> &[u8] {
    match qualified.iter().position(|byte| *byte == b':') {
        Some(index) => &qualified[index + 1..],
        None => qualified,
    }
}

pub fn digest_of(xml: &str) -> Result<String, SignatureError> {
    let canonical = canonicalize_exclusive(xml)?;
    Ok(BASE64.encode(Sha256::digest(canonical.as_bytes())))
}

#[cfg(test)]
mod tests {
    use super::{digest_of, rsa_sha256_signature, sign_enveloped, signed_info};

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

    const CERTIFICATE: &str = "MIICzzCCAbegAwIBAgIU";

    fn assertion() -> String {
        concat!(
            r#"<saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_target">"#,
            "<saml:Issuer>https://idp.example.com</saml:Issuer>",
            "<saml:Subject>alice</saml:Subject>",
            "</saml:Assertion>",
        )
        .to_owned()
    }

    #[test]
    fn the_signature_lands_immediately_after_the_issuer() {
        let signed =
            sign_enveloped(&assertion(), "_target", SIGNING_KEY, CERTIFICATE).expect("sign");
        let after_issuer =
            signed.find("</saml:Issuer>").expect("issuer present") + "</saml:Issuer>".len();
        assert!(signed[after_issuer..].starts_with("<ds:Signature"));
    }

    #[test]
    fn the_signed_document_keeps_everything_it_started_with() {
        let signed =
            sign_enveloped(&assertion(), "_target", SIGNING_KEY, CERTIFICATE).expect("sign");
        assert!(signed.contains("<saml:Subject>alice</saml:Subject>"));
        assert!(signed.contains("<ds:X509Certificate>MIICzzCCAbegAwIBAgIU</ds:X509Certificate>"));
        assert!(signed.contains(r##"<ds:Reference URI="#_target">"##));
    }

    #[test]
    fn the_digest_covers_the_element_as_it_stood_before_signing() {
        let signed =
            sign_enveloped(&assertion(), "_target", SIGNING_KEY, CERTIFICATE).expect("sign");
        let expected = digest_of(&assertion()).expect("digest");
        assert!(signed.contains(&format!("<ds:DigestValue>{expected}</ds:DigestValue>")));
    }

    #[test]
    fn signing_an_already_signed_element_is_refused() {
        let signed =
            sign_enveloped(&assertion(), "_target", SIGNING_KEY, CERTIFICATE).expect("sign");
        assert!(sign_enveloped(&signed, "_target", SIGNING_KEY, CERTIFICATE).is_err());
    }
}

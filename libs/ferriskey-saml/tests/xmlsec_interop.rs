use std::process::Command;

use ferriskey_saml::dsig::sign_enveloped;

const SIGNING_KEY: &str = include_str!("fixtures/signing-key.pem");
const CERTIFICATE_B64: &str = include_str!("fixtures/signing-cert.b64");
const CERTIFICATE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/signing-cert.pem"
);

const ASSERTION_ID_ATTRIBUTE: &str = "--id-attr:ID";
const ASSERTION_QUALIFIED_NAME: &str = "urn:oasis:names:tc:SAML:2.0:assertion:Assertion";

fn assertion() -> String {
    concat!(
        r#"<saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" "#,
        r#"ID="_a1b2c3d4e5f6" Version="2.0" IssueInstant="2026-08-27T10:00:00Z">"#,
        "<saml:Issuer>https://auth.example.com/realms/master</saml:Issuer>",
        "<saml:Subject><saml:NameID>alice@example.com</saml:NameID></saml:Subject>",
        "<saml:AttributeStatement>",
        r#"<saml:Attribute Name="email">"#,
        "<saml:AttributeValue>alice@example.com</saml:AttributeValue>",
        "</saml:Attribute>",
        "</saml:AttributeStatement>",
        "</saml:Assertion>",
    )
    .to_owned()
}

fn verify_with_xmlsec(document: &str) -> std::process::Output {
    let path = std::env::temp_dir().join(format!("ferriskey-saml-{}.xml", document.len()));
    std::fs::write(&path, document).expect("write the document under test");

    Command::new("xmlsec1")
        .arg("--verify")
        .arg("--trusted-pem")
        .arg(CERTIFICATE_PATH)
        .arg(ASSERTION_ID_ATTRIBUTE)
        .arg(ASSERTION_QUALIFIED_NAME)
        .arg(&path)
        .output()
        .expect("run xmlsec1")
}

#[test]
#[ignore = "requires xmlsec1 — run with: cargo test -p ferriskey-saml --test xmlsec_interop -- --ignored"]
fn xmlsec1_accepts_an_assertion_we_signed() {
    let signed = sign_enveloped(
        &assertion(),
        "_a1b2c3d4e5f6",
        SIGNING_KEY,
        CERTIFICATE_B64.trim(),
    )
    .expect("sign the assertion");

    let outcome = verify_with_xmlsec(&signed);

    assert!(
        outcome.status.success(),
        "xmlsec1 rejected an assertion we signed\nstdout: {}\nstderr: {}\ndocument: {}",
        String::from_utf8_lossy(&outcome.stdout),
        String::from_utf8_lossy(&outcome.stderr),
        signed
    );
}

#[test]
#[ignore = "requires xmlsec1 — run with: cargo test -p ferriskey-saml --test xmlsec_interop -- --ignored"]
fn xmlsec1_rejects_an_assertion_whose_content_was_altered_after_signing() {
    let signed = sign_enveloped(
        &assertion(),
        "_a1b2c3d4e5f6",
        SIGNING_KEY,
        CERTIFICATE_B64.trim(),
    )
    .expect("sign the assertion");

    let tampered = signed.replace("alice@example.com", "mallory@example.com");
    assert_ne!(tampered, signed, "the tampering must actually change bytes");

    let outcome = verify_with_xmlsec(&tampered);

    let stderr = String::from_utf8_lossy(&outcome.stderr);

    assert!(
        !outcome.status.success(),
        "xmlsec1 accepted a tampered assertion — the signature covers nothing"
    );
    assert!(
        !stderr.contains("KEY-NOT-FOUND"),
        "xmlsec1 refused for lack of a key rather than a bad signature, \
         so this test proves nothing about the signature: {stderr}"
    );
}

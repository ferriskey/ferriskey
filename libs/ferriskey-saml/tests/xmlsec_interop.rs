use std::process::Command;

use chrono::{DateTime, Utc};
use ferriskey_saml::authn::{AbsoluteUri, Issuer, NameIdFormat, RequestId};
use ferriskey_saml::dsig::sign_enveloped;
use ferriskey_saml::response::{
    AssertionAttribute, AssertionWindow, AttributeNameFormat, AuthnContextClassRef,
    ResponseDescriptor, render_signed_response,
};

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

const RESPONSE_ASSERTION_ID: &str = "_018d3ab762d8fd3f7feb90b5164e821f";

fn instant(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .expect("the fixture instant should parse")
        .with_timezone(&Utc)
}

fn response_descriptor() -> ResponseDescriptor {
    ResponseDescriptor {
        response_id: RequestId::parse("_0d749ed5ea0e6f4800630da48d1f8006")
            .expect("the response id should be an NCName"),
        assertion_id: RequestId::parse(RESPONSE_ASSERTION_ID)
            .expect("the assertion id should be an NCName"),
        in_response_to: RequestId::parse("_324d8ca747d9c07921d2abe9df447832")
            .expect("the request id should be an NCName"),
        issuer: Issuer::parse("https://auth.example.com/realms/master")
            .expect("the issuer should be non empty"),
        destination: AbsoluteUri::parse(
            "Destination",
            "https://chat.example.com/omniauth/saml/callback?account_id=1",
        )
        .expect("the destination should be absolute"),
        audience: AbsoluteUri::parse("Audience", "https://chat.example.com/saml/sp/1")
            .expect("the audience should be absolute"),
        issue_instant: instant("2026-08-25T18:51:18.335Z"),
        authn_instant: instant("2026-08-25T18:51:18.335Z"),
        window: AssertionWindow::new(
            instant("2026-08-25T18:50:18.335Z"),
            instant("2026-08-25T18:56:18.335Z"),
            instant("2026-08-25T19:01:18.335Z"),
        )
        .expect("the reference window should be valid"),
        name_id: "alice@example.com".to_owned(),
        name_id_format: NameIdFormat::EmailAddress,
        session_index: "b1f7c2e4-3d5a-4c8b-9e1f-7a2d6c0b3e59".to_owned(),
        authn_context: AuthnContextClassRef::PasswordProtectedTransport,
        attributes: vec![AssertionAttribute {
            name: "email".to_owned(),
            name_format: AttributeNameFormat::Basic,
            values: vec!["alice@example.com".to_owned()],
        }],
    }
}

#[test]
#[ignore = "requires xmlsec1 — run with: cargo test -p ferriskey-saml --test xmlsec_interop -- --ignored"]
fn xmlsec1_accepts_an_assertion_inside_a_response_we_issued() {
    let signed =
        render_signed_response(&response_descriptor(), SIGNING_KEY, CERTIFICATE_B64.trim())
            .expect("issue a signed response");

    let outcome = verify_with_xmlsec(&signed);

    assert!(
        outcome.status.success(),
        "xmlsec1 rejected an assertion we issued inside a response\nstdout: {}\nstderr: {}\ndocument: {}",
        String::from_utf8_lossy(&outcome.stdout),
        String::from_utf8_lossy(&outcome.stderr),
        signed
    );
}

#[test]
#[ignore = "requires xmlsec1 — run with: cargo test -p ferriskey-saml --test xmlsec_interop -- --ignored"]
fn xmlsec1_accepts_the_assertion_once_it_is_lifted_out_of_the_envelope() {
    let signed =
        render_signed_response(&response_descriptor(), SIGNING_KEY, CERTIFICATE_B64.trim())
            .expect("issue a signed response");

    let start = signed
        .find("<saml:Assertion")
        .expect("the assertion should be present");
    let end = signed
        .find("</saml:Assertion>")
        .expect("the assertion should be closed")
        + "</saml:Assertion>".len();
    let lifted = &signed[start..end];

    let outcome = verify_with_xmlsec(lifted);

    assert!(
        outcome.status.success(),
        "the assertion stopped verifying once detached from the response, so its namespace \
         declaration is not self contained\nstdout: {}\nstderr: {}\ndocument: {}",
        String::from_utf8_lossy(&outcome.stdout),
        String::from_utf8_lossy(&outcome.stderr),
        lifted
    );
}

#[test]
#[ignore = "requires xmlsec1 — run with: cargo test -p ferriskey-saml --test xmlsec_interop -- --ignored"]
fn xmlsec1_rejects_a_response_whose_assertion_was_altered_after_signing() {
    let signed =
        render_signed_response(&response_descriptor(), SIGNING_KEY, CERTIFICATE_B64.trim())
            .expect("issue a signed response");

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

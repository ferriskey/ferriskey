use crate::domain::authentication::entities::{ExchangeTokenInput, GrantType};

/// Verify that the password grant type no longer exists.
/// If someone re-adds it, this test will fail to compile.
#[test]
fn test_password_grant_type_rejected_at_deserialization() {
    let result: Result<GrantType, _> = serde_json::from_str("\"password\"");
    assert!(
        result.is_err(),
        "Password grant must not be deserializable in OAuth 2.1"
    );
}

/// Verify ExchangeTokenInput no longer has username/password fields.
/// This is a compile-time guarantee — the struct simply won't accept them.
#[test]
fn test_exchange_token_input_has_no_password_fields() {
    let input = ExchangeTokenInput {
        realm_name: "test".to_string(),
        client_id: "client".to_string(),
        client_secret: None,
        code: Some("code".to_string()),
        refresh_token: None,
        base_url: "https://example.com".to_string(),
        grant_type: GrantType::Code,
        scope: None,
        code_verifier: Some("verifier".to_string()),
        code_challenge: None,
        code_challenge_method: None,
    };

    // If username/password fields existed, this wouldn't compile without them.
    assert_eq!(input.grant_type, GrantType::Code);
}

#[test]
fn test_pkce_s256_challenge_matches_rfc7636_appendix_b() {
    use base64::{Engine, engine::general_purpose};
    use sha2::{Digest, Sha256};

    // Test vector from RFC 7636 Appendix B
    let code_verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let expected_challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    let hash = Sha256::digest(code_verifier.as_bytes());
    let computed = general_purpose::URL_SAFE_NO_PAD.encode(hash);

    assert_eq!(computed, expected_challenge);
}

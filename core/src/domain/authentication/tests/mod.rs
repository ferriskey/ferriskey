#[cfg(test)]
mod oauth21_tests;

#[cfg(test)]
mod oauth21_compliance_tests {
    use crate::domain::authentication::entities::GrantType;
    use serde_json;

    #[test]
    fn test_grant_type_serialization_oauth21() {
        // Test that only OAuth 2.1 compliant grant types are supported
        let code_grant = GrantType::Code;
        let credentials_grant = GrantType::Credentials;
        let refresh_grant = GrantType::RefreshToken;

        // These should serialize correctly
        assert_eq!(
            serde_json::to_string(&code_grant).unwrap(),
            "\"authorization_code\""
        );
        assert_eq!(
            serde_json::to_string(&credentials_grant).unwrap(),
            "\"client_credentials\""
        );
        assert_eq!(
            serde_json::to_string(&refresh_grant).unwrap(),
            "\"refresh_token\""
        );

        // Test Display implementation
        assert_eq!(code_grant.to_string(), "code");
        assert_eq!(credentials_grant.to_string(), "credentials");
        assert_eq!(refresh_grant.to_string(), "refresh_token");
    }

    #[test]
    fn test_grant_type_deserialization_oauth21() {
        // Test that only OAuth 2.1 compliant grant types deserialize
        let code: GrantType = serde_json::from_str("\"authorization_code\"").unwrap();
        assert!(matches!(code, GrantType::Code));

        let credentials: GrantType = serde_json::from_str("\"client_credentials\"").unwrap();
        assert!(matches!(credentials, GrantType::Credentials));

        let refresh: GrantType = serde_json::from_str("\"refresh_token\"").unwrap();
        assert!(matches!(refresh, GrantType::RefreshToken));

        // Password grant should no longer deserialize (if it existed in JSON)
        // This test would fail if someone tried to use old password grant tokens
        let password_result: Result<GrantType, _> = serde_json::from_str("\"password\"");
        // This should fail because Password variant no longer exists
        assert!(password_result.is_err());
    }

    #[test]
    fn test_pkce_s256_challenge_calculation() {
        use base64::{Engine, engine::general_purpose};
        use sha2::{Digest, Sha256};

        let code_verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

        // Calculate S256 challenge (base64url encoded, no padding)
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let hash = hasher.finalize();

        // Base64 URL encode without padding
        let challenge = general_purpose::URL_SAFE_NO_PAD.encode(hash);

        // Expected challenge for this verifier (base64url encoded)
        let expected_challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        assert_eq!(challenge, expected_challenge);
    }

    #[test]
    fn test_exchange_token_input_has_pkce_fields() {
        use crate::domain::authentication::entities::ExchangeTokenInput;

        // Create exchange token input with PKCE
        let input = ExchangeTokenInput {
            realm_name: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            client_secret: Some("secret".to_string()),
            code: Some("auth_code".to_string()),
            refresh_token: None,
            base_url: "https://auth.example.com".to_string(),
            grant_type: GrantType::Code,
            scope: Some("openid".to_string()),
            code_verifier: Some("verifier".to_string()),
            code_challenge: None,
            code_challenge_method: None,
        };

        assert_eq!(input.realm_name, "test_realm");
        assert_eq!(input.code_verifier, Some("verifier".to_string()));
        assert!(matches!(input.grant_type, GrantType::Code));
    }

    #[test]
    fn test_auth_input_has_pkce_fields() {
        use crate::domain::authentication::entities::AuthInput;

        // Create auth input with PKCE
        let input = AuthInput {
            client_id: "test_client".to_string(),
            realm_name: "test_realm".to_string(),
            redirect_uri: "https://client.com/callback".to_string(),
            response_type: "code".to_string(),
            scope: Some("openid".to_string()),
            state: Some("state".to_string()),
            code_challenge: Some("challenge".to_string()),
            code_challenge_method: Some("S256".to_string()),
        };

        assert_eq!(input.code_challenge, Some("challenge".to_string()));
        assert_eq!(input.code_challenge_method, Some("S256".to_string()));
    }

    #[test]
    fn test_auth_session_params_has_pkce_fields() {
        use crate::domain::authentication::entities::{AuthSession, AuthSessionParams};
        use uuid::Uuid;

        // Create auth session params with PKCE
        let params = AuthSessionParams {
            realm_id: crate::domain::realm::entities::RealmId::from(Uuid::new_v4()),
            client_id: Uuid::new_v4(),
            redirect_uri: "https://client.com/callback".to_string(),
            response_type: "code".to_string(),
            scope: "openid".to_string(),
            state: Some("state".to_string()),
            nonce: None,
            user_id: None,
            code: None,
            authenticated: false,
            webauthn_challenge: None,
            webauthn_challenge_issued_at: None,
            compass_flow_id: None,
            code_challenge: Some("challenge".to_string()),
            code_challenge_method: Some("S256".to_string()),
        };

        let session = AuthSession::new(params);

        assert_eq!(session.code_challenge, Some("challenge".to_string()));
        assert_eq!(session.code_challenge_method, Some("S256".to_string()));
    }

    #[test]
    fn test_grant_type_params_has_pkce_fields() {
        use crate::domain::authentication::value_objects::GrantTypeParams;
        use crate::domain::realm::entities::RealmId;
        use uuid::Uuid;

        // Create grant type params with PKCE
        let params = GrantTypeParams {
            realm_id: RealmId::from(Uuid::new_v4()),
            base_url: "https://auth.example.com".to_string(),
            realm_name: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            client_secret: Some("secret".to_string()),
            code: Some("auth_code".to_string()),
            refresh_token: None,
            redirect_uri: None,
            scope: Some("openid".to_string()),
            code_verifier: Some("verifier".to_string()),
            code_challenge: Some("challenge".to_string()),
            code_challenge_method: Some("S256".to_string()),
        };

        assert_eq!(params.code_verifier, Some("verifier".to_string()));
        assert_eq!(params.code_challenge, Some("challenge".to_string()));
        assert_eq!(params.code_challenge_method, Some("S256".to_string()));
    }
}

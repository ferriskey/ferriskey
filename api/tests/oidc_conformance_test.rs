mod common;

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use jsonwebtoken::{Algorithm, DecodingKey, Validation};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};

    use crate::common::{
        CALLBACK, ClientSpec, SeededClient, TestApp, app, authorize, exchange_code, introspect,
        query_param, rt, sign_in_for_code,
    };

    const PASSWORD: &str = "Conformance-Passw0rd!";

    async fn tokens(app: &TestApp, client: &SeededClient, extra: &[(&str, &str)]) -> Value {
        let server = app.server();
        let user = app.user(&server, PASSWORD).await;
        let code = sign_in_for_code(
            &server,
            app,
            &client.client_id,
            &user.username,
            PASSWORD,
            extra,
        )
        .await;
        let response = exchange_code(&server, app, client, &code, &[]).await;
        assert_eq!(response.status_code(), 200, "token: {}", response.text());
        response.json()
    }

    async fn verify_with_jwks(app: &TestApp, token: &str, audience: &str) -> Value {
        let jwks: Value = app.server().get(&app.oidc("certs")).await.json();
        let header = jsonwebtoken::decode_header(token).expect("jwt header");
        let kid = header.kid.expect("kid in header");
        let key = jwks["keys"]
            .as_array()
            .expect("keys")
            .iter()
            .find(|key| key["kid"] == json!(kid))
            .expect("signing key published in the JWKS");
        let decoding = DecodingKey::from_rsa_components(
            key["n"].as_str().expect("n"),
            key["e"].as_str().expect("e"),
        )
        .expect("rsa key");
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[audience]);
        validation.set_required_spec_claims(&["iss", "aud", "sub", "iat", "exp"]);
        jsonwebtoken::decode::<Value>(token, &decoding, &validation)
            .expect("token verifies against the realm JWKS")
            .claims
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn discovery_endpoints_live_under_the_issuer() {
        rt().block_on(async {
            let app = app();
            let doc: Value = app
                .server()
                .get(&app.path("/.well-known/openid-configuration"))
                .await
                .json();
            let issuer = doc["issuer"].as_str().expect("issuer");
            assert!(
                issuer.ends_with(&format!("/realms/{}", app.realm)),
                "{issuer}"
            );
            for endpoint in [
                "authorization_endpoint",
                "token_endpoint",
                "userinfo_endpoint",
                "jwks_uri",
                "end_session_endpoint",
                "introspection_endpoint",
                "revocation_endpoint",
            ] {
                let url = doc[endpoint].as_str().unwrap_or_default();
                assert!(
                    url.starts_with(issuer),
                    "{endpoint} = {url:?} outside {issuer}"
                );
            }
            assert_eq!(doc["code_challenge_methods_supported"], json!(["S256"]));
            assert!(
                doc["id_token_signing_alg_values_supported"]
                    .as_array()
                    .expect("signing algs")
                    .contains(&json!("RS256"))
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn jwks_publishes_only_public_rsa_material() {
        rt().block_on(async {
            let app = app();
            let jwks: Value = app.server().get(&app.oidc("certs")).await.json();
            let keys = jwks["keys"].as_array().expect("keys");
            assert!(!keys.is_empty(), "the realm publishes a signing key");
            for key in keys {
                assert_eq!(key["kty"], "RSA");
                assert!(key["kid"].is_string(), "kid: {key}");
                assert!(key["n"].is_string() && key["e"].is_string(), "{key}");
                for private in ["d", "p", "q", "dp", "dq", "qi"] {
                    assert!(
                        key.get(private).is_none(),
                        "private member {private} leaked"
                    );
                }
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn id_token_carries_the_oidc_core_claims() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::confidential()).await;
            let body = tokens(app, &client, &[("nonce", "n-0S6_WzA2Mj")]).await;

            let access_token = body["access_token"].as_str().expect("access_token");
            let id_token = body["id_token"]
                .as_str()
                .expect("id_token for scope openid");
            let claims = verify_with_jwks(app, id_token, &client.client_id).await;

            let issuer: Value = app
                .server()
                .get(&app.path("/.well-known/openid-configuration"))
                .await
                .json();
            assert_eq!(claims["iss"], issuer["issuer"]);
            assert_eq!(claims["azp"], json!(client.client_id));
            assert_eq!(claims["nonce"], "n-0S6_WzA2Mj");
            assert!(
                claims["sid"].is_string(),
                "sid for back-channel logout: {claims}"
            );

            let digest = Sha256::digest(access_token.as_bytes());
            let expected_at_hash = URL_SAFE_NO_PAD.encode(&digest[..16]);
            assert_eq!(claims["at_hash"], json!(expected_at_hash));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_redeemed_code_cannot_be_redeemed_again() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            let user = app.user(&server, PASSWORD).await;
            let code = sign_in_for_code(
                &server,
                app,
                &client.client_id,
                &user.username,
                PASSWORD,
                &[],
            )
            .await;

            let first = exchange_code(&server, app, &client, &code, &[]).await;
            assert_eq!(first.status_code(), 200, "{}", first.text());

            let replay = exchange_code(&server, app, &client, &code, &[]).await;
            assert_eq!(replay.status_code(), 400, "{}", replay.text());
            assert_eq!(replay.json::<Value>()["error"], "invalid_grant");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_code_cannot_be_redeemed_by_another_client() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let owner = app.client(ClientSpec::confidential()).await;
            let thief = app.client(ClientSpec::confidential()).await;
            let user = app.user(&server, PASSWORD).await;
            let code = sign_in_for_code(
                &server,
                app,
                &owner.client_id,
                &user.username,
                PASSWORD,
                &[],
            )
            .await;

            let stolen = exchange_code(&server, app, &thief, &code, &[]).await;
            assert!(stolen.status_code().is_client_error(), "{}", stolen.text());

            let legit = exchange_code(&server, app, &owner, &code, &[]).await;
            assert_eq!(legit.status_code(), 200, "{}", legit.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn userinfo_subject_matches_the_id_token() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::confidential()).await;
            let body = tokens(app, &client, &[]).await;
            let id_claims = verify_with_jwks(
                app,
                body["id_token"].as_str().expect("id_token"),
                &client.client_id,
            )
            .await;

            let userinfo = app
                .server()
                .get(&app.oidc("userinfo"))
                .authorization_bearer(body["access_token"].as_str().expect("access_token"))
                .await;
            assert_eq!(userinfo.status_code(), 200, "{}", userinfo.text());
            assert_eq!(userinfo.json::<Value>()["sub"], id_claims["sub"]);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn userinfo_refuses_a_missing_or_id_token_bearer() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::confidential()).await;
            let body = tokens(app, &client, &[]).await;
            let server = app.server();

            let anonymous = server.get(&app.oidc("userinfo")).await;
            assert_eq!(anonymous.status_code(), 401);

            let with_id_token = server
                .get(&app.oidc("userinfo"))
                .authorization_bearer(body["id_token"].as_str().expect("id_token"))
                .await;
            assert_eq!(with_id_token.status_code(), 401, "{}", with_id_token.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn revoking_an_access_token_makes_it_inactive() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            let body = tokens(app, &client, &[]).await;
            let access_token = body["access_token"].as_str().expect("access_token");

            assert_eq!(
                introspect(&server, app, &client, access_token).await["active"],
                true
            );

            let revoked = server
                .post(&app.oidc("revoke"))
                .form(&[
                    ("client_id", client.client_id.as_str()),
                    ("token", access_token),
                ])
                .await;
            assert_eq!(revoked.status_code(), 200, "{}", revoked.text());

            assert_eq!(
                introspect(&server, app, &client, access_token).await["active"],
                false
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn revoking_an_unknown_token_still_answers_200() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::confidential()).await;
            let response = app
                .server()
                .post(&app.oidc("revoke"))
                .form(&[
                    ("client_id", client.client_id.as_str()),
                    ("token", "not-a-token"),
                ])
                .await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn introspecting_garbage_reveals_nothing_but_inactive() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::confidential()).await;
            let body = introspect(&app.server(), app, &client, "not-a-token").await;
            assert_eq!(body, json!({ "active": false }));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn prompt_none_without_a_session_returns_login_required_with_state() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::public()).await;
            let response = authorize(
                &app.server(),
                app,
                &client.client_id,
                &[("prompt", "none"), ("state", "xyz 1")],
            )
            .await;
            assert_eq!(response.status_code(), 302, "{}", response.text());
            let location = response.header("location");
            let location = location.to_str().expect("location header");
            assert!(location.starts_with(CALLBACK), "{location}");
            assert_eq!(
                query_param(location, "error").as_deref(),
                Some("login_required")
            );
            assert_eq!(query_param(location, "state").as_deref(), Some("xyz 1"));
        });
    }
    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_malformed_prompt_is_reported_to_the_registered_redirect_uri() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::public()).await;
            let response = authorize(
                &app.server(),
                app,
                &client.client_id,
                &[("prompt", "none login"), ("state", "s1")],
            )
            .await;
            assert_eq!(response.status_code(), 302, "{}", response.text());
            let location = response.header("location");
            let location = location.to_str().expect("location header");
            assert!(location.starts_with(CALLBACK), "{location}");
            assert_eq!(
                query_param(location, "error").as_deref(),
                Some("invalid_request")
            );
            assert_eq!(query_param(location, "state").as_deref(), Some("s1"));
        });
    }
}

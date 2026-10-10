mod common;

#[cfg(test)]
mod tests {
    use base64::{
        Engine,
        engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    };
    use jsonwebtoken::{Algorithm, DecodingKey, Validation};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};

    use crate::common::{
        ADMIN_CLIENT_ID, CALLBACK, ClientSpec, SeededClient, TestApp, app, authorize,
        client_credentials, exchange_code, introspect, password_grant, query_param, refresh, rt,
        sign_in_for_code,
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
    fn a_code_is_not_redeemed_for_a_user_disabled_meanwhile() {
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
            sqlx::query("UPDATE users SET enabled = false WHERE id = $1::uuid")
                .bind(&user.id)
                .execute(&app.pool)
                .await
                .expect("disable user");

            let response = exchange_code(&server, app, &client, &code, &[]).await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], "invalid_grant");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn maintenance_is_checked_on_the_client_of_the_authorization_request() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let closed = app.client(ClientSpec::public()).await;
            let open = app.client(ClientSpec::public()).await;
            let user = app.user(&server, PASSWORD).await;
            sqlx::query("UPDATE clients SET maintenance_enabled = true WHERE id = $1")
                .bind(closed.id)
                .execute(&app.pool)
                .await
                .expect("enable maintenance");

            let authorized = authorize(&server, app, &closed.client_id, &[]).await;
            assert_eq!(authorized.status_code(), 302, "{}", authorized.text());

            let response = server
                .post(&app.path("/login-actions/authenticate"))
                .add_cookie(authorized.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", &open.client_id)
                .json(&json!({ "username": user.username, "password": PASSWORD }))
                .await;

            let body = response.text();
            assert_ne!(response.status_code(), 200, "{body}");
            assert!(!body.contains("code="), "{body}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_verifier_is_refused_for_a_code_issued_without_pkce() {
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

            let response = exchange_code(
                &server,
                app,
                &client,
                &code,
                &[(
                    "code_verifier",
                    "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk",
                )],
            )
            .await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], "invalid_grant");
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
                    (
                        "client_secret",
                        client.secret.as_deref().unwrap_or_default(),
                    ),
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
                    (
                        "client_secret",
                        client.secret.as_deref().unwrap_or_default(),
                    ),
                    ("token", "not-a-token"),
                ])
                .await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_login_step_token_is_not_an_active_token() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            let user = app.user(&server, PASSWORD).await;
            sqlx::query(
                "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data, user_label)
                 VALUES ($1, 'otp', $2::uuid, 'JBSWY3DPEHPK3PXP', '{}'::jsonb, 'test-authenticator')",
            )
            .bind(uuid::Uuid::new_v4())
            .bind(&user.id)
            .execute(&app.pool)
            .await
            .expect("insert otp credential");

            let authorized = authorize(&server, app, &client.client_id, &[]).await;
            let login = server
                .post(&app.path("/login-actions/authenticate"))
                .add_cookie(authorized.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", &client.client_id)
                .json(&json!({ "username": user.username, "password": PASSWORD }))
                .await;
            assert_eq!(login.json::<Value>()["status"], "RequiresOtpChallenge");
            let step = login.cookie("FERRISKEY_LOGIN_ACTION").value().to_string();

            let body = introspect(&server, app, &client, &step).await;
            assert_eq!(body["active"], false, "{body}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_rotated_refresh_token_is_not_active() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            let issued = tokens(app, &client, &[]).await;
            let first = issued["refresh_token"].as_str().expect("refresh_token");

            let rotated = refresh(
                &server,
                app,
                &client.client_id,
                client.secret.as_deref(),
                first,
            )
            .await;
            assert_eq!(rotated.status_code(), 200, "{}", rotated.text());

            let body = introspect(&server, app, &client, first).await;
            assert_eq!(body["active"], false, "{body}");
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

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn logout_refuses_an_id_token_hint_with_a_forged_signature() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::confidential()).await;
            app.add_post_logout_redirect_uri(&client, "https://app.example/out")
                .await;
            let body = tokens(app, &client, &[]).await;
            let id_token = body["id_token"].as_str().expect("id_token");
            let (unsigned, signature) = id_token.rsplit_once('.').expect("jwt");
            let flipped = if signature.starts_with('A') { "B" } else { "A" };
            let forged = format!("{unsigned}.{flipped}{}", &signature[1..]);

            let response = app
                .server()
                .get(&app.oidc("logout"))
                .add_query_param("id_token_hint", &forged)
                .add_query_param("post_logout_redirect_uri", "https://app.example/out")
                .await;
            let location = response
                .maybe_header("location")
                .and_then(|value| value.to_str().ok().map(str::to_string))
                .unwrap_or_default();
            assert!(
                !location.starts_with("https://app.example/out"),
                "forged hint accepted: {} {location}",
                response.status_code()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn concurrent_redemptions_of_one_code_yield_one_token_set() {
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

            let (first, second) = tokio::join!(
                exchange_code(&server, app, &client, &code, &[]),
                exchange_code(&server, app, &client, &code, &[]),
            );
            let successes = [first.status_code(), second.status_code()]
                .iter()
                .filter(|status| status.as_u16() == 200)
                .count();
            assert_eq!(successes, 1, "both redemptions of one code succeeded");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn parallel_wrong_passwords_lock_the_account() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let user = app.user(&server, PASSWORD).await;
            sqlx::query(
                "UPDATE realm_settings SET lockout_threshold = 5, lockout_duration_seconds = 900 WHERE realm_id = $1",
            )
            .bind(app.realm_id)
            .execute(&app.pool)
            .await
            .expect("configure lockout");

            let wrong =
                || password_grant(&server, app, ADMIN_CLIENT_ID, None, &user.username, "wrong");
            tokio::join!(wrong(), wrong(), wrong(), wrong(), wrong());

            let after = password_grant(
                &server,
                app,
                ADMIN_CLIENT_ID,
                None,
                &user.username,
                PASSWORD,
            )
            .await;
            assert_ne!(
                after.status_code(),
                200,
                "five parallel failures did not lock the account"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn wrong_otp_codes_lock_the_account_across_logins() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::public()).await;
            let user = app.user(&server, PASSWORD).await;
            sqlx::query(
                "UPDATE realm_settings SET lockout_threshold = 5, lockout_duration_seconds = 900 WHERE realm_id = $1",
            )
            .bind(app.realm_id)
            .execute(&app.pool)
            .await
            .expect("configure lockout");
            sqlx::query(
                "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data, user_label)
                 VALUES ($1, 'otp', $2::uuid, 'JBSWY3DPEHPK3PXP', '{}'::jsonb, 'test-authenticator')",
            )
            .bind(uuid::Uuid::new_v4())
            .bind(&user.id)
            .execute(&app.pool)
            .await
            .expect("insert otp credential");

            for _ in 0..2 {
                let authorized = authorize(&server, app, &client.client_id, &[]).await;
                let session = authorized.cookie("FERRISKEY_SESSION").value().to_string();
                let login = server
                    .post(&app.path("/login-actions/authenticate"))
                    .add_cookie(authorized.cookie("FERRISKEY_SESSION"))
                    .add_query_param("client_id", &client.client_id)
                    .json(&json!({ "username": user.username, "password": PASSWORD }))
                    .await;
                assert_eq!(login.status_code(), 200, "{}", login.text());
                assert_eq!(login.json::<Value>()["status"], "RequiresOtpChallenge");
                let step = login.cookie("FERRISKEY_LOGIN_ACTION").value().to_string();

                for _ in 0..3 {
                    let challenge = server
                        .post(&app.path("/login-actions/challenge-otp"))
                        .add_header(
                            "Cookie",
                            format!("FERRISKEY_LOGIN_ACTION={step}; FERRISKEY_SESSION={session}"),
                        )
                        .json(&json!({ "code": "000000" }))
                        .await;
                    assert_ne!(challenge.status_code(), 200, "{}", challenge.text());
                }
            }

            let locked: bool = sqlx::query_scalar(
                "SELECT locked_until IS NOT NULL AND locked_until > NOW() FROM users WHERE id = $1::uuid",
            )
            .bind(&user.id)
            .fetch_one(&app.pool)
            .await
            .expect("read lock");
            assert!(locked, "six wrong OTP codes did not lock the account");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_long_scope_is_not_a_server_error() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::public()).await;
            let scope = format!("openid {}", "urn:example:api:read ".repeat(20));
            let response = app
                .server()
                .get(&app.oidc("auth"))
                .add_query_param("response_type", "code")
                .add_query_param("client_id", &client.client_id)
                .add_query_param("redirect_uri", CALLBACK)
                .add_query_param("scope", scope.trim_end())
                .await;
            assert!(
                response.status_code().as_u16() < 500,
                "{}: {}",
                response.status_code(),
                response.text()
            );
        });
    }
    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_confidential_password_grant_requires_the_secret() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app
                .client(ClientSpec::confidential().with_direct_access())
                .await;
            let user = app.user(&server, PASSWORD).await;

            let response = password_grant(
                &server,
                app,
                &client.client_id,
                None,
                &user.username,
                PASSWORD,
            )
            .await;
            assert_eq!(response.status_code(), 401, "{}", response.text());
        });
    }
    #[test]
    #[ignore = "requires PostgreSQL"]
    fn revoke_accepts_basic_client_authentication() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            let body = tokens(app, &client, &[]).await;
            let access_token = body["access_token"]
                .as_str()
                .expect("access_token")
                .to_string();
            let credentials = STANDARD.encode(format!(
                "{}:{}",
                client.client_id,
                client.secret.as_deref().unwrap_or_default()
            ));

            let revoked = server
                .post(&app.oidc("revoke"))
                .add_header("authorization", format!("Basic {credentials}"))
                .form(&[("token", access_token.as_str())])
                .await;
            assert_eq!(revoked.status_code(), 200, "{}", revoked.text());
            assert_eq!(
                introspect(&server, app, &client, &access_token).await["active"],
                false
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_confidential_client_cannot_revoke_without_its_secret() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            let body = tokens(app, &client, &[]).await;
            let access_token = body["access_token"].as_str().expect("access_token");

            let refused = server
                .post(&app.oidc("revoke"))
                .form(&[
                    ("client_id", client.client_id.as_str()),
                    ("token", access_token),
                ])
                .await;
            assert_eq!(refused.status_code(), 401, "{}", refused.text());
            assert_eq!(refused.json::<Value>()["error"], "invalid_client");
            assert_eq!(
                introspect(&server, app, &client, access_token).await["active"],
                true
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn client_credentials_issue_no_refresh_token() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app
                .client(ClientSpec::confidential().with_service_account())
                .await;
            app.bind_service_account(&server, &client).await;

            let response = client_credentials(&server, app, &client).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let body: Value = response.json();
            assert!(body["access_token"].is_string(), "{body}");
            assert!(body.get("refresh_token").is_none(), "{body}");
            assert!(body.get("refresh_expires_in").is_none(), "{body}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_public_client_cannot_use_client_credentials() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app
                .client(ClientSpec::public().with_service_account())
                .await;
            app.bind_service_account(&server, &client).await;

            let response = client_credentials(&server, app, &client).await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], "unauthorized_client");
        });
    }
    #[test]
    #[ignore = "requires PostgreSQL"]
    fn an_access_token_is_not_a_login_for_another_client() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let issuing = app.client(ClientSpec::confidential()).await;
            let target = app.client(ClientSpec::public()).await;
            let access_token = tokens(app, &issuing, &[]).await["access_token"]
                .as_str()
                .expect("access_token")
                .to_string();

            let authorized = authorize(&server, app, &target.client_id, &[]).await;
            assert_eq!(authorized.status_code(), 302, "{}", authorized.text());

            let response = server
                .post(&app.path("/login-actions/authenticate"))
                .add_cookie(authorized.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", &target.client_id)
                .authorization_bearer(&access_token)
                .json(&json!({}))
                .await;

            let body = response.text();
            assert_eq!(response.status_code(), 400, "{body}");
            assert!(!body.contains("code="), "{body}");
            assert!(
                response.maybe_cookie("FERRISKEY_SSO").is_none(),
                "an SSO cookie was issued: {body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_disabled_service_account_cannot_use_client_credentials() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app
                .client(ClientSpec::confidential().with_service_account())
                .await;
            app.bind_service_account(&server, &client).await;
            sqlx::query("UPDATE users SET enabled = false WHERE client_id = $1")
                .bind(client.id)
                .execute(&app.pool)
                .await
                .expect("disable service account");

            let response = client_credentials(&server, app, &client).await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], "unauthorized_client");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn token_response_is_not_cacheable() {
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

            let response = exchange_code(&server, app, &client, &code, &[]).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let cache_control = response
                .maybe_header("cache-control")
                .and_then(|value| value.to_str().ok().map(str::to_string));
            assert_eq!(cache_control.as_deref(), Some("no-store"));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_missing_code_is_an_invalid_request() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::confidential()).await;
            let response = app
                .server()
                .post(&app.oidc("token"))
                .form(&[
                    ("grant_type", "authorization_code"),
                    ("client_id", client.client_id.as_str()),
                    (
                        "client_secret",
                        client.secret.as_deref().unwrap_or_default(),
                    ),
                ])
                .await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], "invalid_request");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_wrong_hint_still_revokes_the_token() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            let body = tokens(app, &client, &[]).await;
            let access_token = body["access_token"]
                .as_str()
                .expect("access_token")
                .to_string();

            let revoked = server
                .post(&app.oidc("revoke"))
                .form(&[
                    ("client_id", client.client_id.as_str()),
                    (
                        "client_secret",
                        client.secret.as_deref().unwrap_or_default(),
                    ),
                    ("token", access_token.as_str()),
                    ("token_type_hint", "refresh_token"),
                ])
                .await;
            assert_eq!(revoked.status_code(), 200, "{}", revoked.text());
            assert_eq!(
                introspect(&server, app, &client, &access_token).await["active"],
                false
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn an_unknown_grant_type_is_unsupported() {
        rt().block_on(async {
            let app = app();
            let response = app
                .server()
                .post(&app.oidc("token"))
                .form(&[("grant_type", "urn:example:made-up"), ("client_id", "x")])
                .await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], "unsupported_grant_type");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_wrong_password_is_an_invalid_grant() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let user = app.user(&server, PASSWORD).await;
            let response =
                password_grant(&server, app, ADMIN_CLIENT_ID, None, &user.username, "wrong").await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], "invalid_grant");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn an_unknown_client_is_an_invalid_client() {
        rt().block_on(async {
            let app = app();
            let response = app
                .server()
                .post(&app.oidc("token"))
                .form(&[
                    ("grant_type", "client_credentials"),
                    ("client_id", "no-such-client"),
                    ("client_secret", "whatever"),
                ])
                .await;
            assert_eq!(response.status_code(), 401, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], "invalid_client");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_forged_refresh_token_is_an_invalid_grant() {
        rt().block_on(async {
            let app = app();
            let client = app.client(ClientSpec::public()).await;
            let forged = format!(
                "{}.{}.{}",
                URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256","typ":"JWT"}"#),
                URL_SAFE_NO_PAD.encode(r#"{"sub":"x","exp":1}"#),
                URL_SAFE_NO_PAD.encode("signature")
            );
            let response = app
                .server()
                .post(&app.oidc("token"))
                .form(&[
                    ("grant_type", "refresh_token"),
                    ("client_id", client.client_id.as_str()),
                    ("refresh_token", forged.as_str()),
                ])
                .await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], "invalid_grant");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn token_errors_are_not_cacheable() {
        rt().block_on(async {
            let app = app();
            let response = app
                .server()
                .post(&app.oidc("token"))
                .form(&[("grant_type", "authorization_code"), ("client_id", "x")])
                .await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            let cache_control = response
                .maybe_header("cache-control")
                .and_then(|value| value.to_str().ok().map(str::to_string));
            assert_eq!(cache_control.as_deref(), Some("no-store"));
        });
    }
}

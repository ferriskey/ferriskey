mod common;

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde_json::Value;

    use crate::common::{
        ADMIN_CLIENT_ID, CALLBACK, ClientSpec, SeededClient, TestApp, app, exchange_code,
        introspect, password_grant, refresh, rt, sign_in_for_code,
    };

    const PASSWORD: &str = "Findings-Passw0rd!";

    async fn signed_in(app: &TestApp, client: &SeededClient) -> Value {
        let server = app.server();
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
        let response = exchange_code(&server, app, client, &code, &[]).await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        response.json()
    }

    fn field(body: &Value, name: &str) -> String {
        body[name]
            .as_str()
            .unwrap_or_else(|| panic!("{name} missing from {body}"))
            .to_string()
    }

    #[test]
    #[ignore = "SSO-01 open: the code is spent after the tokens are minted"]
    fn sso_01_concurrent_redemptions_yield_one_token_set() {
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
    #[ignore = "SSO-06 open: failed_login_attempts is a read-modify-write"]
    fn sso_06_parallel_wrong_passwords_lock_the_account() {
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
    #[ignore = "SSO-08 open: a concurrent refresh revokes the whole family"]
    fn sso_08_parallel_refreshes_keep_the_session_alive() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            let tokens = signed_in(app, &client).await;
            let refresh_token = field(&tokens, "refresh_token");
            let secret = client.secret.as_deref();

            let (first, second) = tokio::join!(
                refresh(&server, app, &client.client_id, secret, &refresh_token),
                refresh(&server, app, &client.client_id, secret, &refresh_token),
            );
            let survivor = [first, second]
                .into_iter()
                .find(|response| response.status_code() == 200)
                .expect("one of the parallel refreshes succeeds");
            let next = field(&survivor.json::<Value>(), "refresh_token");

            let follow_up = refresh(&server, app, &client.client_id, secret, &next).await;
            assert_eq!(
                follow_up.status_code(),
                200,
                "the winning refresh token was revoked: {}",
                follow_up.text()
            );
        });
    }

    #[test]
    #[ignore = "SSO-12 open: token responses lack Cache-Control: no-store"]
    fn sso_12_token_response_is_not_cacheable() {
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
    #[ignore = "SSO-12 open: a missing code is an internal server error"]
    fn sso_12_missing_code_is_an_invalid_request() {
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
    #[ignore = "SSO-13 open: /revoke ignores HTTP Basic client authentication"]
    fn sso_13_revoke_accepts_basic_client_authentication() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            let tokens = signed_in(app, &client).await;
            let access_token = field(&tokens, "access_token");
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
    #[ignore = "SSO-13 open: a wrong token_type_hint skips the revocation"]
    fn sso_13_a_wrong_hint_still_revokes_the_token() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            let tokens = signed_in(app, &client).await;
            let access_token = field(&tokens, "access_token");

            let revoked = server
                .post(&app.oidc("revoke"))
                .form(&[
                    ("client_id", client.client_id.as_str()),
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
    #[ignore = "SSO-11 open: the issuer is derived from the Host header"]
    fn sso_11_host_header_does_not_change_the_issuer() {
        rt().block_on(async {
            let app = app();
            let doc: Value = app
                .server()
                .get(&app.path("/.well-known/openid-configuration"))
                .add_header("host", "evil.example")
                .await
                .json();
            let issuer = doc["issuer"].as_str().expect("issuer");
            assert!(!issuer.contains("evil.example"), "issuer = {issuer}");
        });
    }

    #[test]
    #[ignore = "SSO-21 open: a confidential client may skip its secret on the password grant"]
    fn sso_21_confidential_password_grant_requires_the_secret() {
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
    #[ignore = "SSO-22 open: auth_sessions.scope is VARCHAR(255)"]
    fn sso_22_a_long_scope_is_not_a_server_error() {
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
}

mod common;

#[cfg(test)]
mod tests {
    use axum_extra::extract::cookie::Cookie;
    use axum_test::{TestResponse, TestServer};
    use chrono::{DateTime, Duration, NaiveDateTime, SubsecRound, Utc};
    use serde_json::json;
    use uuid::Uuid;

    use crate::common::{
        CALLBACK, ClientSpec, SeededClient, TestApp, app, authorize, query_param, rt,
    };

    const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    const OTHER_CHALLENGE: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

    struct Login {
        cookie: Cookie<'static>,
        session_id: Uuid,
    }

    async fn identity_provider(app: &TestApp, server: &TestServer) -> String {
        let token = app.admin_token(server).await;
        let alias = format!("idp-{}", Uuid::new_v4().simple());
        let response = server
            .post(&app.path("/identity-providers"))
            .add_header("Authorization", format!("Bearer {token}"))
            .json(&json!({
                "alias": alias,
                "provider_id": "oidc",
                "enabled": true,
                "display_name": "Continue with Google",
                "store_token": false,
                "add_read_token_role_on_create": false,
                "trust_email": false,
                "link_only": false,
                "config": {
                    "client_id": "google-client",
                    "client_secret": "google-secret",
                    "authorization_url": "https://idp.example/authorize",
                    "token_url": "https://idp.example/token",
                    "scopes": ["openid", "email"],
                    "use_pkce": false,
                },
            }))
            .await;
        assert_eq!(
            response.status_code(),
            201,
            "create identity provider: {}",
            response.text()
        );
        alias
    }

    async fn open_login(
        app: &TestApp,
        server: &TestServer,
        client: &SeededClient,
        state: &str,
    ) -> Login {
        let response = authorize(
            server,
            app,
            &client.client_id,
            &[
                ("state", state),
                ("code_challenge", CHALLENGE),
                ("code_challenge_method", "S256"),
            ],
        )
        .await;
        assert_eq!(
            response.status_code(),
            302,
            "authorize: {}",
            response.text()
        );
        let cookie = response.cookie("FERRISKEY_SESSION");
        let session_id = Uuid::parse_str(cookie.value()).expect("session cookie is a uuid");
        Login { cookie, session_id }
    }

    async fn press_provider_button(
        app: &TestApp,
        server: &TestServer,
        alias: &str,
        client: &SeededClient,
        state: &str,
        cookie: Option<&Login>,
        extra: &[(&str, &str)],
    ) -> TestResponse {
        let mut request = server
            .get(&app.path(&format!("/broker/{alias}/login")))
            .add_query_param("client_id", &client.client_id)
            .add_query_param("redirect_uri", CALLBACK)
            .add_query_param("state", state);
        for (key, value) in extra {
            request = request.add_query_param(key, value);
        }
        if let Some(login) = cookie {
            request = request.add_cookie(login.cookie.clone());
        }
        request.await
    }

    fn broker_state_of(response: &TestResponse) -> String {
        assert_eq!(
            response.status_code(),
            302,
            "the provider button did not redirect: {}",
            response.text()
        );
        let location = response
            .headers()
            .get("location")
            .expect("Location header")
            .to_str()
            .expect("Location is utf-8")
            .to_string();
        query_param(&location, "state").expect("broker state in Location")
    }

    async fn binding_of(app: &TestApp, response: &TestResponse) -> (Option<Uuid>, Option<String>) {
        sqlx::query_as(
            "SELECT auth_session_id, code_challenge FROM broker_auth_sessions \
             WHERE broker_state = $1",
        )
        .bind(broker_state_of(response))
        .fetch_one(&app.pool)
        .await
        .expect("broker session")
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test broker_login_pkce_test -- --ignored"]
    fn a_provider_login_resumes_the_session_that_holds_the_pkce_challenge() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let alias = identity_provider(app, &server).await;
            let client = app.client(ClientSpec::public()).await;
            let login = open_login(app, &server, &client, "state-1").await;

            let response =
                press_provider_button(app, &server, &alias, &client, "state-1", Some(&login), &[])
                    .await;

            let (auth_session_id, _) = binding_of(app, &response).await;
            assert_eq!(auth_session_id, Some(login.session_id));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test broker_login_pkce_test -- --ignored"]
    fn a_provider_login_of_a_client_requiring_pkce_resumes_the_session_that_holds_it() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let alias = identity_provider(app, &server).await;
            let client = app.client(ClientSpec::public().with_required_pkce()).await;
            let login = open_login(app, &server, &client, "state-2").await;

            let response =
                press_provider_button(app, &server, &alias, &client, "state-2", Some(&login), &[])
                    .await;

            let (auth_session_id, _) = binding_of(app, &response).await;
            assert_eq!(auth_session_id, Some(login.session_id));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test broker_login_pkce_test -- --ignored"]
    fn a_provider_login_of_a_client_requiring_pkce_is_refused_without_the_session_cookie() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let alias = identity_provider(app, &server).await;
            let client = app.client(ClientSpec::public().with_required_pkce()).await;
            open_login(app, &server, &client, "state-3").await;

            let response =
                press_provider_button(app, &server, &alias, &client, "state-3", None, &[]).await;

            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(
                response.json::<serde_json::Value>()["error"],
                "invalid_request"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test broker_login_pkce_test -- --ignored"]
    fn a_provider_login_does_not_resume_the_session_of_another_client() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let alias = identity_provider(app, &server).await;
            let opener = app.client(ClientSpec::public()).await;
            let other = app.client(ClientSpec::public()).await;
            let login = open_login(app, &server, &opener, "state-4").await;

            let response =
                press_provider_button(app, &server, &alias, &other, "state-4", Some(&login), &[])
                    .await;

            let (auth_session_id, _) = binding_of(app, &response).await;
            assert_eq!(auth_session_id, None);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test broker_login_pkce_test -- --ignored"]
    fn a_provider_login_does_not_resume_a_session_opened_for_another_state() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let alias = identity_provider(app, &server).await;
            let client = app.client(ClientSpec::public()).await;
            let login = open_login(app, &server, &client, "state-5").await;

            let response = press_provider_button(
                app,
                &server,
                &alias,
                &client,
                "another-state",
                Some(&login),
                &[],
            )
            .await;

            let (auth_session_id, _) = binding_of(app, &response).await;
            assert_eq!(auth_session_id, None);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test broker_login_pkce_test -- --ignored"]
    fn a_provider_login_that_brings_its_own_challenge_ignores_the_session_cookie() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let alias = identity_provider(app, &server).await;
            let client = app.client(ClientSpec::public()).await;
            let login = open_login(app, &server, &client, "state-6").await;

            let response = press_provider_button(
                app,
                &server,
                &alias,
                &client,
                "state-6",
                Some(&login),
                &[
                    ("code_challenge", OTHER_CHALLENGE),
                    ("code_challenge_method", "S256"),
                ],
            )
            .await;

            let (auth_session_id, code_challenge) = binding_of(app, &response).await;
            assert_eq!(auth_session_id, None);
            assert_eq!(code_challenge.as_deref(), Some(OTHER_CHALLENGE));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test broker_login_pkce_test -- --ignored"]
    fn a_session_named_in_the_query_is_not_resumed_without_its_cookie() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let alias = identity_provider(app, &server).await;
            let client = app.client(ClientSpec::public()).await;
            let planted = open_login(app, &server, &client, "state-7").await;
            let planted_id = planted.session_id.to_string();

            let response = press_provider_button(
                app,
                &server,
                &alias,
                &client,
                "state-7",
                None,
                &[("session_id", planted_id.as_str())],
            )
            .await;

            let (auth_session_id, _) = binding_of(app, &response).await;
            assert_eq!(auth_session_id, None);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test broker_login_pkce_test -- --ignored"]
    fn a_session_named_in_the_query_does_not_replace_the_session_of_the_cookie() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let alias = identity_provider(app, &server).await;
            let client = app.client(ClientSpec::public()).await;
            let planted = open_login(app, &server, &client, "state-8").await;
            let own = open_login(app, &server, &client, "state-8").await;
            let planted_id = planted.session_id.to_string();

            let response = press_provider_button(
                app,
                &server,
                &alias,
                &client,
                "state-8",
                Some(&own),
                &[("session_id", planted_id.as_str())],
            )
            .await;

            let (auth_session_id, _) = binding_of(app, &response).await;
            assert_eq!(auth_session_id, Some(own.session_id));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test broker_login_pkce_test -- --ignored"]
    fn the_login_page_receives_the_redirect_uri_and_state_intact() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let alias = identity_provider(app, &server).await;
            let redirect_uri = "http://localhost/callback?tenant=a&lang=fr";
            let state = "a+b/c=d&e";
            let client = app
                .client(ClientSpec {
                    redirect_uris: vec![redirect_uri.to_string()],
                    ..ClientSpec::public()
                })
                .await;

            let authorized = server
                .get(&app.oidc("auth"))
                .add_query_param("response_type", "code")
                .add_query_param("client_id", &client.client_id)
                .add_query_param("redirect_uri", redirect_uri)
                .add_query_param("scope", "openid")
                .add_query_param("state", state)
                .add_query_param("code_challenge", CHALLENGE)
                .add_query_param("code_challenge_method", "S256")
                .await;
            assert_eq!(
                authorized.status_code(),
                302,
                "authorize: {}",
                authorized.text()
            );
            let cookie = authorized.cookie("FERRISKEY_SESSION");
            let session_id = Uuid::parse_str(cookie.value()).expect("session cookie is a uuid");
            let login_page = authorized
                .headers()
                .get("location")
                .expect("Location header")
                .to_str()
                .expect("Location is utf-8")
                .to_string();

            let page_redirect_uri = query_param(&login_page, "redirect_uri");
            let page_state = query_param(&login_page, "state");
            assert_eq!(
                page_redirect_uri.as_deref(),
                Some(redirect_uri),
                "{login_page}"
            );
            assert_eq!(page_state.as_deref(), Some(state), "{login_page}");

            let response = server
                .get(&app.path(&format!("/broker/{alias}/login")))
                .add_query_param("client_id", &client.client_id)
                .add_query_param("redirect_uri", redirect_uri)
                .add_query_param("state", state)
                .add_cookie(cookie)
                .await;

            let (auth_session_id, _) = binding_of(app, &response).await;
            assert_eq!(auth_session_id, Some(session_id));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test broker_login_pkce_test -- --ignored"]
    fn the_broker_session_expires_with_the_session_it_resumes() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let alias = identity_provider(app, &server).await;
            let client = app.client(ClientSpec::public()).await;
            let login = open_login(app, &server, &client, "state-9").await;
            let soon: NaiveDateTime = (Utc::now() + Duration::minutes(2))
                .naive_utc()
                .trunc_subsecs(6);
            sqlx::query("UPDATE auth_sessions SET expires_at = $1 WHERE id = $2")
                .bind(soon)
                .bind(login.session_id)
                .execute(&app.pool)
                .await
                .expect("shorten the auth session");

            let response =
                press_provider_button(app, &server, &alias, &client, "state-9", Some(&login), &[])
                    .await;

            let broker_expiry: DateTime<Utc> = sqlx::query_scalar(
                "SELECT expires_at FROM broker_auth_sessions WHERE broker_state = $1",
            )
            .bind(broker_state_of(&response))
            .fetch_one(&app.pool)
            .await
            .expect("broker session");
            assert_eq!(broker_expiry, soon.and_utc());
        });
    }
}

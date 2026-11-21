mod common;

#[cfg(test)]
mod tests {
    use axum_extra::extract::cookie::Cookie;
    use axum_test::{TestResponse, TestServer};
    use serde_json::{Value, json};

    use crate::common::{
        CALLBACK, ClientSpec, SeededClient, TestApp, app, authorize, decode_unverified,
        exchange_code, query_param, rt,
    };

    const PASSWORD: &str = "Registration-Passw0rd!";

    static REALM_SETTINGS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    fn fresh_email() -> String {
        format!("resumed-{}@example.com", uuid::Uuid::new_v4().simple())
    }

    fn username_of(email: &str) -> &str {
        email.trim_end_matches("@example.com")
    }

    async fn open_the_flow(
        server: &TestServer,
        app: &TestApp,
        client: &SeededClient,
    ) -> Cookie<'static> {
        open_the_flow_with(server, app, client, &[]).await
    }

    async fn open_the_flow_with(
        server: &TestServer,
        app: &TestApp,
        client: &SeededClient,
        extra: &[(&str, &str)],
    ) -> Cookie<'static> {
        let mut params = vec![("prompt", "create"), ("state", "s-resume")];
        params.extend_from_slice(extra);
        let authorized = authorize(server, app, &client.client_id, &params).await;
        assert_eq!(authorized.status_code(), 302, "{}", authorized.text());
        authorized.cookie("FERRISKEY_SESSION")
    }

    async fn register_in(
        server: &TestServer,
        app: &TestApp,
        session: &Cookie<'static>,
        email: &str,
    ) -> TestResponse {
        server
            .post(&app.oidc("registrations"))
            .add_cookie(session.clone())
            .json(&json!({
                "username": username_of(email),
                "email": email,
                "password": PASSWORD,
            }))
            .await
    }

    async fn register_inside_the_flow(
        server: &TestServer,
        app: &TestApp,
        client: &SeededClient,
        email: &str,
    ) -> TestResponse {
        let session = open_the_flow(server, app, client).await;
        register_in(server, app, &session, email).await
    }

    async fn users_registered_with(app: &TestApp, email: &str) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1 AND realm_id = $2")
            .bind(email)
            .bind(app.realm_id)
            .fetch_one(&app.pool)
            .await
            .expect("count users")
    }

    async fn set_require_mfa(app: &TestApp, required: bool) {
        sqlx::query("UPDATE realm_settings SET require_mfa = $1 WHERE realm_id = $2")
            .bind(required)
            .bind(app.realm_id)
            .execute(&app.pool)
            .await
            .expect("set require mfa");
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_registration_hands_the_sso_cookie_to_the_browser() {
        rt().block_on(async {
            let _settings = REALM_SETTINGS.lock().await;
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::public()).await;
            app.set_user_registration(true).await;

            let registered = register_inside_the_flow(&server, app, &client, &fresh_email()).await;

            assert_eq!(registered.status_code(), 201, "{}", registered.text());
            let body: Value = registered.json();
            assert_eq!(body["status"], "redirect", "{body}");
            let cookie = registered
                .maybe_cookie("FERRISKEY_SSO")
                .expect("the new SSO session reaches the browser");
            assert!(
                !registered.text().contains(cookie.value()),
                "the SSO secret must stay out of the body: {}",
                registered.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_registration_for_a_client_in_maintenance_is_refused_and_creates_no_account() {
        rt().block_on(async {
            let _settings = REALM_SETTINGS.lock().await;
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::public()).await;
            app.set_user_registration(true).await;
            sqlx::query("UPDATE clients SET maintenance_enabled = true WHERE id = $1")
                .bind(client.id)
                .execute(&app.pool)
                .await
                .expect("enable maintenance");

            let email = fresh_email();
            let registered = register_inside_the_flow(&server, app, &client, &email).await;

            let body = registered.text();
            assert_eq!(registered.status_code(), 503, "{body}");
            assert!(!body.contains("code="), "{body}");
            assert!(!body.contains(CALLBACK), "{body}");
            assert_eq!(
                users_registered_with(app, &email).await,
                0,
                "a refused registration must leave no account behind"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_registration_owing_mfa_enrollment_does_not_resume_the_flow() {
        rt().block_on(async {
            let _settings = REALM_SETTINGS.lock().await;
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::public()).await;
            app.set_user_registration(true).await;
            set_require_mfa(app, true).await;

            let registered = register_inside_the_flow(&server, app, &client, &fresh_email()).await;

            set_require_mfa(app, false).await;
            assert_eq!(registered.status_code(), 201, "{}", registered.text());
            let body: Value = registered.json();
            assert_eq!(body["status"], "pending_action", "{body}");
            assert!(!body.to_string().contains("code="), "{body}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_registration_owing_mfa_enrollment_leaves_the_flow_open_for_the_next_sign_in() {
        rt().block_on(async {
            let _settings = REALM_SETTINGS.lock().await;
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::public()).await;
            app.set_user_registration(true).await;
            set_require_mfa(app, true).await;

            let email = fresh_email();
            let session = open_the_flow(&server, app, &client).await;
            let registered = register_in(&server, app, &session, &email).await;
            let signed_in = server
                .post(&app.path("/login-actions/authenticate"))
                .add_cookie(session)
                .add_query_param("client_id", &client.client_id)
                .json(&json!({ "username": username_of(&email), "password": PASSWORD }))
                .await;

            set_require_mfa(app, false).await;
            assert_eq!(registered.status_code(), 201, "{}", registered.text());
            assert_eq!(signed_in.status_code(), 200, "{}", signed_in.text());
            let body: Value = signed_in.json();
            assert_eq!(body["status"], "RequiresActions", "{body}");
            assert_eq!(body["required_actions"], json!(["configure_otp"]), "{body}");
            assert!(
                signed_in.maybe_cookie("FERRISKEY_LOGIN_ACTION").is_some(),
                "the sign-in must hand over the step token that finishes the enrollment"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_registration_for_a_multi_factor_request_does_not_resume_the_flow() {
        rt().block_on(async {
            let _settings = REALM_SETTINGS.lock().await;
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::public()).await;
            app.set_user_registration(true).await;

            let session = open_the_flow_with(&server, app, &client, &[("acr_values", "2")]).await;
            let registered = register_in(&server, app, &session, &fresh_email()).await;

            assert_eq!(registered.status_code(), 201, "{}", registered.text());
            let body: Value = registered.json();
            assert_eq!(body["status"], "pending_action", "{body}");
            assert!(!body.to_string().contains("code="), "{body}");
            assert!(registered.maybe_cookie("FERRISKEY_SSO").is_none());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_registration_answers_an_essential_single_factor_acr() {
        rt().block_on(async {
            let _settings = REALM_SETTINGS.lock().await;
            let app = app();
            let server = app.server();
            let client = app.client(ClientSpec::confidential()).await;
            app.set_user_registration(true).await;
            let claims = r#"{"id_token":{"acr":{"essential":true,"values":["1"]}}}"#;

            let session = open_the_flow_with(&server, app, &client, &[("claims", claims)]).await;
            let registered = register_in(&server, app, &session, &fresh_email()).await;
            assert_eq!(registered.status_code(), 201, "{}", registered.text());
            let body: Value = registered.json();
            assert_eq!(body["status"], "redirect", "{body}");
            let url = body["data"]["url"].as_str().expect("a redirect url");
            let code = query_param(url, "code").expect("an authorization code");

            let token = exchange_code(&server, app, &client, &code, &[]).await;
            assert_eq!(token.status_code(), 200, "{}", token.text());
            let id_token = decode_unverified(
                token.json::<Value>()["id_token"]
                    .as_str()
                    .expect("an id token"),
            );

            assert_eq!(id_token["acr"], "1", "{id_token}");
            assert_eq!(id_token["amr"], json!(["pwd"]), "{id_token}");
        });
    }
}

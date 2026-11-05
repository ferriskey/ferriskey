mod common;

#[cfg(test)]
mod tests {
    use axum_test::{TestResponse, TestServer};
    use serde_json::{Value, json};

    use crate::common::{CALLBACK, ClientSpec, SeededClient, TestApp, app, authorize, rt};

    const PASSWORD: &str = "Registration-Passw0rd!";

    static REALM_SETTINGS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    fn fresh_email() -> String {
        format!("resumed-{}@example.com", uuid::Uuid::new_v4().simple())
    }

    async fn register_inside_the_flow(
        server: &TestServer,
        app: &TestApp,
        client: &SeededClient,
        email: &str,
    ) -> TestResponse {
        let authorized = authorize(
            server,
            app,
            &client.client_id,
            &[("prompt", "create"), ("state", "s-resume")],
        )
        .await;
        assert_eq!(authorized.status_code(), 302, "{}", authorized.text());

        server
            .post(&app.oidc("registrations"))
            .add_cookie(authorized.cookie("FERRISKEY_SESSION"))
            .json(&json!({
                "username": email.trim_end_matches("@example.com"),
                "email": email,
                "password": PASSWORD,
            }))
            .await
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
}

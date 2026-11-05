mod common;

#[cfg(test)]
mod tests {
    use axum_test::{TestResponse, TestServer};
    use serde_json::{Value, json};

    use crate::common::{CALLBACK, ClientSpec, SeededClient, TestApp, app, authorize, rt};

    const PASSWORD: &str = "Registration-Passw0rd!";

    static REALM_SETTINGS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    async fn register_inside_the_flow(
        server: &TestServer,
        app: &TestApp,
        client: &SeededClient,
    ) -> TestResponse {
        let authorized = authorize(
            server,
            app,
            &client.client_id,
            &[("prompt", "create"), ("state", "s-resume")],
        )
        .await;
        assert_eq!(authorized.status_code(), 302, "{}", authorized.text());

        let suffix = uuid::Uuid::new_v4().simple().to_string();
        server
            .post(&app.oidc("registrations"))
            .add_cookie(authorized.cookie("FERRISKEY_SESSION"))
            .json(&json!({
                "username": format!("resumed-{suffix}"),
                "email": format!("resumed-{suffix}@example.com"),
                "password": PASSWORD,
            }))
            .await
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

            let registered = register_inside_the_flow(&server, app, &client).await;

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
    fn a_registration_for_a_client_in_maintenance_does_not_resume_the_flow() {
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

            let registered = register_inside_the_flow(&server, app, &client).await;

            let body = registered.text();
            assert_ne!(registered.status_code(), 201, "{body}");
            assert!(!body.contains("code="), "{body}");
            assert!(!body.contains(CALLBACK), "{body}");
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

            let registered = register_inside_the_flow(&server, app, &client).await;

            set_require_mfa(app, false).await;
            assert_eq!(registered.status_code(), 201, "{}", registered.text());
            let body: Value = registered.json();
            assert_eq!(body["status"], "pending_action", "{body}");
            assert!(!body.to_string().contains("code="), "{body}");
        });
    }
}

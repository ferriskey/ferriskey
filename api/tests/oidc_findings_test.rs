mod common;

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::common::{
        ClientSpec, SeededClient, TestApp, app, exchange_code, refresh, rt, sign_in_for_code,
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
}

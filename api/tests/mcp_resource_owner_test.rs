mod common;

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, MutexGuard};

    use axum_test::{TestResponse, TestServer};
    use serde_json::{Value, json};
    use uuid::Uuid;

    use crate::common::{
        ClientSpec, SeededClient, TestApp, app, decode_unverified, exchange_code, password_grant,
        query_param, rt, sign_in_for_redirect,
    };

    const PASSWORD: &str = "Resource-Owner-Passw0rd!";
    const EXCHANGE: &str = "urn:ietf:params:oauth:grant-type:token-exchange";
    const ACCESS_TOKEN_TYPE: &str = "urn:ietf:params:oauth:token-type:access_token";

    static SERIAL: Mutex<()> = Mutex::new(());

    fn serial() -> MutexGuard<'static, ()> {
        SERIAL
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn resource() -> String {
        format!("https://mcp-{}.example.com/mcp", Uuid::new_v4().simple())
    }

    async fn exchanger(app: &TestApp) -> SeededClient {
        let client = app
            .client(ClientSpec::confidential().with_direct_access())
            .await;
        sqlx::query("UPDATE clients SET token_exchange_enabled = true WHERE id = $1")
            .bind(client.id)
            .execute(&app.pool)
            .await
            .expect("enable token exchange");
        client
    }

    async fn allow_policy(
        server: &TestServer,
        app: &TestApp,
        admin: &str,
        client: &SeededClient,
        target: &SeededClient,
    ) {
        let response = server
            .post(&app.path(&format!("/clients/{}/token-exchange-policies", client.id)))
            .authorization_bearer(admin)
            .json(&json!({
                "target_audience": target.client_id,
                "allow_impersonation": true,
                "allow_delegation": true,
            }))
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());
    }

    async fn set_allowed_resources(
        server: &TestServer,
        app: &TestApp,
        admin: &str,
        resources: &[&str],
    ) -> TestResponse {
        server
            .put(&app.path("/settings"))
            .authorization_bearer(admin)
            .json(&json!({ "allowed_resources": resources }))
            .await
    }

    async fn put_owners(
        server: &TestServer,
        app: &TestApp,
        admin: &str,
        owners: &[(&str, &SeededClient)],
    ) -> TestResponse {
        let owners: Vec<Value> = owners
            .iter()
            .map(|(uri, client)| json!({ "uri": uri, "client_id": client.id }))
            .collect();
        server
            .put(&app.path("/resource-owners"))
            .authorization_bearer(admin)
            .json(&json!({ "owners": owners }))
            .await
    }

    async fn subject_token(
        server: &TestServer,
        app: &TestApp,
        frontend: &SeededClient,
        resource: &str,
    ) -> String {
        let user = app.user(server, PASSWORD).await;
        let url = sign_in_for_redirect(
            server,
            app,
            &frontend.client_id,
            &user.username,
            PASSWORD,
            &[("resource", resource)],
        )
        .await;
        let code = query_param(&url, "code").expect("authorization code");
        let response = exchange_code(server, app, frontend, &code, &[]).await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        let token = response.json::<Value>()["access_token"]
            .as_str()
            .expect("access token")
            .to_string();
        let claims = decode_unverified(&token);
        assert_eq!(claims["aud"], json!([resource]));
        assert_eq!(claims["azp"], frontend.client_id);
        token
    }

    async fn actor_token(server: &TestServer, app: &TestApp, client: &SeededClient) -> String {
        let user = app.user(server, PASSWORD).await;
        let response = password_grant(
            server,
            app,
            &client.client_id,
            client.secret.as_deref(),
            &user.username,
            PASSWORD,
        )
        .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        response.json::<Value>()["access_token"]
            .as_str()
            .expect("actor token")
            .to_string()
    }

    async fn token_exchange(
        server: &TestServer,
        app: &TestApp,
        client: &SeededClient,
        subject: &str,
        audience: &str,
        actor: Option<&str>,
    ) -> TestResponse {
        let mut form = vec![
            ("grant_type", EXCHANGE),
            ("client_id", client.client_id.as_str()),
            (
                "client_secret",
                client.secret.as_deref().unwrap_or_default(),
            ),
            ("subject_token", subject),
            ("subject_token_type", ACCESS_TOKEN_TYPE),
            ("audience", audience),
        ];
        if let Some(actor) = actor {
            form.push(("actor_token", actor));
            form.push(("actor_token_type", ACCESS_TOKEN_TYPE));
        }
        server.post(&app.oidc("token")).form(&form).await
    }

    fn assert_unauthorized_client(response: &TestResponse) {
        assert_eq!(response.status_code(), 400, "{}", response.text());
        assert_eq!(
            response.json::<Value>()["error"],
            "unauthorized_client",
            "{}",
            response.text()
        );
    }

    struct Scenario {
        server: TestServer,
        admin: String,
        resource: String,
        frontend: SeededClient,
        owner: SeededClient,
        outsider: SeededClient,
        target: SeededClient,
        subject: String,
        actor: String,
    }

    async fn scenario(app: &TestApp) -> Scenario {
        let server = app.server();
        let admin = app.admin_token(&server).await;
        let resource = resource();

        let settings = set_allowed_resources(&server, app, &admin, &[&resource]).await;
        assert_eq!(settings.status_code(), 200, "{}", settings.text());

        let frontend = app.client(ClientSpec::confidential()).await;
        let owner = exchanger(app).await;
        let outsider = exchanger(app).await;
        let target = app.client(ClientSpec::confidential()).await;
        allow_policy(&server, app, &admin, &owner, &target).await;
        allow_policy(&server, app, &admin, &outsider, &target).await;

        let subject = subject_token(&server, app, &frontend, &resource).await;
        let actor = actor_token(&server, app, &owner).await;

        Scenario {
            server,
            admin,
            resource,
            frontend,
            owner,
            outsider,
            target,
            subject,
            actor,
        }
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_owner_of_a_resource_exchanges_a_token_issued_to_another_client_for_it() {
        let _guard = serial();
        rt().block_on(async {
            let app = app();
            let s = scenario(app).await;

            let before = token_exchange(
                &s.server,
                app,
                &s.owner,
                &s.subject,
                &s.target.client_id,
                None,
            )
            .await;
            assert_unauthorized_client(&before);

            let owners = put_owners(&s.server, app, &s.admin, &[(&s.resource, &s.owner)]).await;
            assert_eq!(owners.status_code(), 200, "{}", owners.text());
            assert_eq!(
                owners.json::<Value>(),
                json!({ "data": [{ "uri": s.resource, "client_id": s.owner.id }] })
            );
            let listed = s
                .server
                .get(&app.path("/resource-owners"))
                .authorization_bearer(&s.admin)
                .await;
            assert_eq!(listed.status_code(), 200, "{}", listed.text());
            assert_eq!(listed.json::<Value>(), owners.json::<Value>());

            let plain = token_exchange(
                &s.server,
                app,
                &s.owner,
                &s.subject,
                &s.target.client_id,
                None,
            )
            .await;
            assert_eq!(plain.status_code(), 200, "{}", plain.text());
            let claims = decode_unverified(
                plain.json::<Value>()["access_token"]
                    .as_str()
                    .expect("access token"),
            );
            assert_eq!(claims["aud"], json!([s.target.client_id]));
            assert_eq!(claims["azp"], s.owner.client_id);

            let delegated = token_exchange(
                &s.server,
                app,
                &s.owner,
                &s.subject,
                &s.target.client_id,
                Some(&s.actor),
            )
            .await;
            assert_eq!(delegated.status_code(), 200, "{}", delegated.text());
            let claims = decode_unverified(
                delegated.json::<Value>()["access_token"]
                    .as_str()
                    .expect("access token"),
            );
            assert_eq!(claims["aud"], json!([s.target.client_id]));
            assert_eq!(claims["act"]["client_id"], s.owner.client_id);

            let outsider = token_exchange(
                &s.server,
                app,
                &s.outsider,
                &s.subject,
                &s.target.client_id,
                None,
            )
            .await;
            assert_unauthorized_client(&outsider);

            let frontend_party = token_exchange(
                &s.server,
                app,
                &s.frontend,
                &s.subject,
                &s.target.client_id,
                None,
            )
            .await;
            assert_ne!(frontend_party.status_code(), 200);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn removing_the_ownership_or_the_resource_stops_the_exchange() {
        let _guard = serial();
        rt().block_on(async {
            let app = app();
            let s = scenario(app).await;

            let owners = put_owners(&s.server, app, &s.admin, &[(&s.resource, &s.owner)]).await;
            assert_eq!(owners.status_code(), 200, "{}", owners.text());
            let works = token_exchange(
                &s.server,
                app,
                &s.owner,
                &s.subject,
                &s.target.client_id,
                None,
            )
            .await;
            assert_eq!(works.status_code(), 200, "{}", works.text());

            let cleared = put_owners(&s.server, app, &s.admin, &[]).await;
            assert_eq!(cleared.status_code(), 200, "{}", cleared.text());
            assert_eq!(cleared.json::<Value>(), json!({ "data": [] }));
            let refused = token_exchange(
                &s.server,
                app,
                &s.owner,
                &s.subject,
                &s.target.client_id,
                None,
            )
            .await;
            assert_unauthorized_client(&refused);

            let owners = put_owners(&s.server, app, &s.admin, &[(&s.resource, &s.owner)]).await;
            assert_eq!(owners.status_code(), 200, "{}", owners.text());
            let again = token_exchange(
                &s.server,
                app,
                &s.owner,
                &s.subject,
                &s.target.client_id,
                None,
            )
            .await;
            assert_eq!(again.status_code(), 200, "{}", again.text());

            let removed = set_allowed_resources(&s.server, app, &s.admin, &[]).await;
            assert_eq!(removed.status_code(), 200, "{}", removed.text());
            let (rows,): (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM realm_resource_owners WHERE realm_id = $1")
                    .bind(app.realm_id)
                    .fetch_one(&app.pool)
                    .await
                    .expect("owner rows");
            assert_eq!(rows, 0, "the owner row goes with the resource");

            let gone = token_exchange(
                &s.server,
                app,
                &s.owner,
                &s.subject,
                &s.target.client_id,
                None,
            )
            .await;
            assert_unauthorized_client(&gone);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn only_confidential_admin_clients_can_own_an_allowed_resource_once() {
        let _guard = serial();
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let admin = app.admin_token(&server).await;
            let resource = resource();
            let other = self::resource();
            let settings = set_allowed_resources(&server, app, &admin, &[&resource]).await;
            assert_eq!(settings.status_code(), 200, "{}", settings.text());

            let public = app.client(ClientSpec::public()).await;
            let dynamic = app.client(ClientSpec::confidential()).await;
            sqlx::query("UPDATE clients SET registration_source = 'dynamic' WHERE id = $1")
                .bind(dynamic.id)
                .execute(&app.pool)
                .await
                .expect("mark dynamic");
            let confidential = app.client(ClientSpec::confidential()).await;

            let refused_public = put_owners(&server, app, &admin, &[(&resource, &public)]).await;
            assert_eq!(
                refused_public.status_code(),
                400,
                "{}",
                refused_public.text()
            );

            let refused_dynamic = put_owners(&server, app, &admin, &[(&resource, &dynamic)]).await;
            assert_eq!(
                refused_dynamic.status_code(),
                400,
                "{}",
                refused_dynamic.text()
            );

            let refused_uri = put_owners(&server, app, &admin, &[(&other, &confidential)]).await;
            assert_eq!(refused_uri.status_code(), 400, "{}", refused_uri.text());

            let refused_twice = put_owners(
                &server,
                app,
                &admin,
                &[(&resource, &confidential), (&resource, &confidential)],
            )
            .await;
            assert_eq!(refused_twice.status_code(), 400, "{}", refused_twice.text());

            let unknown = server
                .put(&app.path("/resource-owners"))
                .authorization_bearer(&admin)
                .json(&json!({ "owners": [{ "uri": resource, "client_id": Uuid::new_v4() }] }))
                .await;
            assert_eq!(unknown.status_code(), 400, "{}", unknown.text());

            let (rows,): (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM realm_resource_owners WHERE realm_id = $1")
                    .bind(app.realm_id)
                    .fetch_one(&app.pool)
                    .await
                    .expect("owner rows");
            assert_eq!(rows, 0, "a refused replacement changes nothing");

            let accepted = put_owners(&server, app, &admin, &[(&resource, &confidential)]).await;
            assert_eq!(accepted.status_code(), 200, "{}", accepted.text());
            put_owners(&server, app, &admin, &[]).await;
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_string_list_settings_payload_still_works_and_gives_resources_without_owner() {
        let _guard = serial();
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let admin = app.admin_token(&server).await;
            let resource = resource();

            let settings = set_allowed_resources(&server, app, &admin, &[&resource]).await;
            assert_eq!(settings.status_code(), 200, "{}", settings.text());
            assert_eq!(
                settings.json::<Value>()["data"]["settings"]["allowed_resources"],
                json!([resource])
            );

            let owners = server
                .get(&app.path("/resource-owners"))
                .authorization_bearer(&admin)
                .await;
            assert_eq!(owners.status_code(), 200, "{}", owners.text());
            assert_eq!(owners.json::<Value>(), json!({ "data": [] }));

            let anonymous = server.get(&app.path("/resource-owners")).await;
            assert_eq!(anonymous.status_code(), 401);
        });
    }
}

mod common;

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::common::{
        CALLBACK, ClientSpec, SeededClient, TestApp, app, authorize, decode_unverified,
        exchange_code, query_param, refresh, rt, sign_in_for_redirect,
    };

    const PASSWORD: &str = "Mcp-Protocol-Passw0rd!";
    const MCP_RESOURCE: &str = "https://mcp.example.com/mcp";
    const OTHER_RESOURCE: &str = "https://other.example.com/api";
    const UNKNOWN_RESOURCE: &str = "https://unknown.example.com/";

    async fn prepare(app: &TestApp) -> SeededClient {
        app.set_allowed_resources(&[MCP_RESOURCE, OTHER_RESOURCE])
            .await;
        app.client(ClientSpec::confidential()).await
    }

    async fn redirect_for(app: &TestApp, client: &SeededClient, extra: &[(&str, &str)]) -> String {
        let server = app.server();
        let user = app.user(&server, PASSWORD).await;
        sign_in_for_redirect(
            &server,
            app,
            &client.client_id,
            &user.username,
            PASSWORD,
            extra,
        )
        .await
    }

    async fn issuer_of(app: &TestApp) -> String {
        let document: Value = app
            .server()
            .get(&app.path("/.well-known/openid-configuration"))
            .await
            .json();
        document["issuer"].as_str().expect("issuer").to_string()
    }

    fn strings(document: &Value, key: &str) -> Vec<String> {
        document[key]
            .as_array()
            .unwrap_or_else(|| panic!("{key} missing in {document}"))
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect()
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn authorization_server_metadata_lists_the_rfc_8414_fields_and_follows_the_toggles() {
        rt().block_on(async {
            let app = app();
            let server = app.server();
            let url = format!(
                "/.well-known/oauth-authorization-server/realms/{}",
                app.realm
            );

            app.ensure_client_scope("mcp-protocol-scope").await;
            app.set_registration_toggles(false, false).await;
            let response = server.get(&url).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let document: Value = response.json();
            eprintln!(
                "RFC8414 (toggles off): {}",
                serde_json::to_string_pretty(&document).expect("json")
            );

            let issuer = document["issuer"].as_str().expect("issuer");
            assert!(
                issuer.ends_with(&format!("/realms/{}", app.realm)),
                "{issuer}"
            );
            assert_eq!(
                document["authorization_endpoint"],
                json!(format!("{issuer}/protocol/openid-connect/auth"))
            );
            assert_eq!(
                document["token_endpoint"],
                json!(format!("{issuer}/protocol/openid-connect/token"))
            );
            assert_eq!(
                document["jwks_uri"],
                json!(format!("{issuer}/protocol/openid-connect/jwks.json"))
            );
            assert_eq!(
                document["revocation_endpoint"],
                json!(format!("{issuer}/protocol/openid-connect/revoke"))
            );
            assert_eq!(
                document["introspection_endpoint"],
                json!(format!("{issuer}/protocol/openid-connect/token/introspect"))
            );
            assert_eq!(strings(&document, "response_types_supported"), ["code"]);
            assert_eq!(
                strings(&document, "code_challenge_methods_supported"),
                ["S256"]
            );
            assert_eq!(
                document["authorization_response_iss_parameter_supported"],
                json!(true)
            );
            assert!(
                strings(&document, "scopes_supported")
                    .iter()
                    .any(|scope| scope == "mcp-protocol-scope")
            );

            let grants = strings(&document, "grant_types_supported");
            for grant in [
                "authorization_code",
                "refresh_token",
                "client_credentials",
                "password",
                "urn:ietf:params:oauth:grant-type:device_code",
                "urn:ietf:params:oauth:grant-type:token-exchange",
            ] {
                assert!(grants.iter().any(|g| g == grant), "{grant} in {grants:?}");
            }

            let methods = strings(&document, "token_endpoint_auth_methods_supported");
            assert_eq!(
                methods,
                ["none", "client_secret_basic", "client_secret_post"]
            );

            assert!(document.get("registration_endpoint").is_none());
            assert!(
                document
                    .get("client_id_metadata_document_supported")
                    .is_none()
            );

            app.set_registration_toggles(true, false).await;
            let document: Value = server.get(&url).await.json();
            assert_eq!(
                document["registration_endpoint"],
                json!(format!("{issuer}/clients/register"))
            );
            assert!(
                document
                    .get("client_id_metadata_document_supported")
                    .is_none()
            );

            app.set_registration_toggles(false, true).await;
            let document: Value = server.get(&url).await.json();
            assert!(document.get("registration_endpoint").is_none());
            assert_eq!(
                document["client_id_metadata_document_supported"],
                json!(true)
            );

            app.set_registration_toggles(true, true).await;
            let document: Value = server.get(&url).await.json();
            eprintln!(
                "RFC8414 (toggles on): {}",
                serde_json::to_string_pretty(&document).expect("json")
            );
            assert!(document.get("registration_endpoint").is_some());
            assert_eq!(
                document["client_id_metadata_document_supported"],
                json!(true)
            );

            let openid: Value = server
                .get(&app.path("/.well-known/openid-configuration"))
                .await
                .json();
            eprintln!(
                "OpenID (toggles on): {}",
                serde_json::to_string_pretty(&openid).expect("json")
            );
            assert_eq!(openid["issuer"], document["issuer"]);
            assert_eq!(
                openid["registration_endpoint"],
                document["registration_endpoint"]
            );
            assert_eq!(openid["client_id_metadata_document_supported"], json!(true));
            assert_eq!(
                openid["authorization_response_iss_parameter_supported"],
                json!(true)
            );
            assert_eq!(openid["scopes_supported"], document["scopes_supported"]);
            assert_eq!(
                strings(&openid, "token_endpoint_auth_methods_supported"),
                methods
            );

            app.set_registration_toggles(false, false).await;
            let openid: Value = server
                .get(&app.path("/.well-known/openid-configuration"))
                .await
                .json();
            assert!(openid.get("registration_endpoint").is_none());
            assert!(
                openid
                    .get("client_id_metadata_document_supported")
                    .is_none()
            );

            let unknown = server
                .get("/.well-known/oauth-authorization-server/realms/no-such-realm")
                .await;
            assert!(unknown.status_code().is_client_error());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_success_redirect_carries_code_state_and_iss() {
        rt().block_on(async {
            let app = app();
            let client = prepare(app).await;
            let issuer = issuer_of(app).await;

            let url = redirect_for(app, &client, &[("state", "st ate-1")]).await;

            assert!(url.starts_with(CALLBACK), "{url}");
            assert!(query_param(&url, "code").is_some(), "{url}");
            assert_eq!(query_param(&url, "state").as_deref(), Some("st ate-1"));
            assert_eq!(query_param(&url, "iss"), Some(issuer));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn error_redirects_carry_iss() {
        rt().block_on(async {
            let app = app();
            let client = prepare(app).await;
            let server = app.server();
            let issuer = issuer_of(app).await;

            for (extra, error) in [
                (vec![("prompt", "none"), ("state", "s1")], "login_required"),
                (
                    vec![("prompt", "none login"), ("state", "s1")],
                    "invalid_request",
                ),
                (
                    vec![("resource", UNKNOWN_RESOURCE), ("state", "s1")],
                    "invalid_target",
                ),
            ] {
                let response = authorize(&server, app, &client.client_id, &extra).await;
                assert_eq!(response.status_code(), 302, "{error}");
                let location = response
                    .header("location")
                    .to_str()
                    .expect("location")
                    .to_string();
                assert!(location.starts_with(CALLBACK), "{location}");
                assert_eq!(query_param(&location, "error").as_deref(), Some(error));
                assert_eq!(query_param(&location, "state").as_deref(), Some("s1"));
                assert_eq!(
                    query_param(&location, "iss").as_deref(),
                    Some(issuer.as_str()),
                    "{location}"
                );
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_access_token_audience_is_exactly_the_requested_resource() {
        rt().block_on(async {
            let app = app();
            let client = prepare(app).await;
            let server = app.server();

            let url = redirect_for(app, &client, &[("resource", MCP_RESOURCE)]).await;
            let code = query_param(&url, "code").expect("code");
            let response = exchange_code(&server, app, &client, &code, &[]).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let tokens: Value = response.json();

            let claims = decode_unverified(tokens["access_token"].as_str().expect("access"));
            assert_eq!(claims["aud"], json!([MCP_RESOURCE]));

            let url = redirect_for(app, &client, &[("resource", MCP_RESOURCE)]).await;
            let code = query_param(&url, "code").expect("code");
            let response =
                exchange_code(&server, app, &client, &code, &[("resource", MCP_RESOURCE)]).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let claims = decode_unverified(
                response.json::<Value>()["access_token"]
                    .as_str()
                    .expect("access"),
            );
            assert_eq!(claims["aud"], json!([MCP_RESOURCE]));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_code_without_a_resource_accepts_an_allowed_one_at_the_token_endpoint() {
        rt().block_on(async {
            let app = app();
            let client = prepare(app).await;
            let server = app.server();

            let url = redirect_for(app, &client, &[]).await;
            let code = query_param(&url, "code").expect("code");
            let response = exchange_code(
                &server,
                app,
                &client,
                &code,
                &[("resource", OTHER_RESOURCE)],
            )
            .await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let claims = decode_unverified(
                response.json::<Value>()["access_token"]
                    .as_str()
                    .expect("access"),
            );
            assert_eq!(claims["aud"], json!([OTHER_RESOURCE]));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn without_a_resource_the_audience_is_unchanged() {
        rt().block_on(async {
            let app = app();
            let client = prepare(app).await;
            let server = app.server();

            let url = redirect_for(app, &client, &[]).await;
            let code = query_param(&url, "code").expect("code");
            let response = exchange_code(&server, app, &client, &code, &[]).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let claims = decode_unverified(
                response.json::<Value>()["access_token"]
                    .as_str()
                    .expect("access"),
            );
            let audience = claims["aud"].to_string();
            assert!(audience.contains("account"), "{audience}");
            assert!(!audience.contains(MCP_RESOURCE), "{audience}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_token_endpoint_refuses_a_resource_that_is_not_the_bound_one() {
        rt().block_on(async {
            let app = app();
            let client = prepare(app).await;
            let server = app.server();

            let url = redirect_for(app, &client, &[("resource", MCP_RESOURCE)]).await;
            let code = query_param(&url, "code").expect("code");
            let response = exchange_code(
                &server,
                app,
                &client,
                &code,
                &[("resource", OTHER_RESOURCE)],
            )
            .await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], json!("invalid_target"));

            let response = exchange_code(&server, app, &client, &code, &[]).await;
            assert_eq!(
                response.status_code(),
                200,
                "the code is not burned by a refusal"
            );

            let url = redirect_for(app, &client, &[]).await;
            let code = query_param(&url, "code").expect("code");
            let response = exchange_code(
                &server,
                app,
                &client,
                &code,
                &[("resource", UNKNOWN_RESOURCE)],
            )
            .await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_eq!(response.json::<Value>()["error"], json!("invalid_target"));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_refresh_keeps_the_original_resource_and_refuses_another() {
        rt().block_on(async {
            let app = app();
            let client = prepare(app).await;
            let server = app.server();

            let url = redirect_for(app, &client, &[("resource", MCP_RESOURCE)]).await;
            let code = query_param(&url, "code").expect("code");
            let tokens: Value = exchange_code(&server, app, &client, &code, &[])
                .await
                .json();
            let refresh_token = tokens["refresh_token"]
                .as_str()
                .expect("refresh")
                .to_string();

            let other = server
                .post(&app.oidc("token"))
                .form(&[
                    ("grant_type", "refresh_token"),
                    ("client_id", client.client_id.as_str()),
                    (
                        "client_secret",
                        client.secret.as_deref().unwrap_or_default(),
                    ),
                    ("refresh_token", refresh_token.as_str()),
                    ("resource", OTHER_RESOURCE),
                ])
                .await;
            assert_eq!(other.status_code(), 400, "{}", other.text());
            assert_eq!(other.json::<Value>()["error"], json!("invalid_target"));

            let refreshed = refresh(
                &server,
                app,
                &client.client_id,
                client.secret.as_deref(),
                &refresh_token,
            )
            .await;
            assert_eq!(refreshed.status_code(), 200, "{}", refreshed.text());
            let refreshed: Value = refreshed.json();
            let claims = decode_unverified(refreshed["access_token"].as_str().expect("access"));
            assert_eq!(claims["aud"], json!([MCP_RESOURCE]));

            let next = refreshed["refresh_token"].as_str().expect("refresh");
            let again = server
                .post(&app.oidc("token"))
                .form(&[
                    ("grant_type", "refresh_token"),
                    ("client_id", client.client_id.as_str()),
                    (
                        "client_secret",
                        client.secret.as_deref().unwrap_or_default(),
                    ),
                    ("refresh_token", next),
                    ("resource", MCP_RESOURCE),
                ])
                .await;
            assert_eq!(again.status_code(), 200, "{}", again.text());
            let claims = decode_unverified(
                again.json::<Value>()["access_token"]
                    .as_str()
                    .expect("access"),
            );
            assert_eq!(claims["aud"], json!([MCP_RESOURCE]));
        });
    }
}

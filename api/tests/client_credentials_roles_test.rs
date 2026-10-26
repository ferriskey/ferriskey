mod common;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use common::{ClientSpec, SeededClient, TestApp, app, cached_app, client_credentials, rt};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

async fn service_account(app: &TestApp) -> (SeededClient, Uuid) {
    let server = app.server();
    let client = app
        .client(ClientSpec::confidential().with_service_account())
        .await;
    app.bind_service_account(&server, &client).await;

    let scope_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO client_scopes (id, realm_id, name, description, protocol, default_scope_type, created_at, updated_at) \
         VALUES ($1, $2, $3, $3, 'openid-connect', 'DEFAULT', now(), now())",
    )
    .bind(scope_id)
    .bind(app.realm_id)
    .bind(format!("roles-{}", scope_id.simple()))
    .execute(&app.pool)
    .await
    .expect("insert the role scope");
    for mapper_type in [
        "oidc-usermodel-realm-role-mapper",
        "oidc-usermodel-client-role-mapper",
    ] {
        sqlx::query(
            "INSERT INTO client_scope_protocol_mappers (id, client_scope_id, name, mapper_type, config, created_at) \
             VALUES ($1, $2, $3, $3, '{\"access.token.claim\": \"true\", \"id.token.claim\": \"false\"}'::jsonb, now())",
        )
        .bind(Uuid::new_v4())
        .bind(scope_id)
        .bind(mapper_type)
        .execute(&app.pool)
        .await
        .expect("insert the role mapper");
    }
    sqlx::query(
        "INSERT INTO client_scope_mappings (client_id, client_scope_id, default_scope_type) VALUES ($1, $2, 'DEFAULT')",
    )
    .bind(client.id)
    .bind(scope_id)
    .execute(&app.pool)
    .await
    .expect("attach the role scope");

    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE client_id = $1")
        .bind(client.id)
        .fetch_one(&app.pool)
        .await
        .expect("service account user");
    (client, user_id)
}

async fn role(pool: &PgPool, realm_id: Uuid, client_id: Option<Uuid>, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO roles (id, name, realm_id, client_id) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(name)
        .bind(realm_id)
        .bind(client_id)
        .execute(pool)
        .await
        .expect("insert role");
    id
}

async fn assign(pool: &PgPool, user_id: Uuid, role_id: Uuid) {
    sqlx::query("INSERT INTO user_role (user_id, role_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(role_id)
        .execute(pool)
        .await
        .expect("assign role");
}

async fn group(pool: &PgPool, organization_id: Uuid, parent: Option<Uuid>, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO organization_groups (id, organization_id, parent_group_id, name) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(organization_id)
    .bind(parent)
    .bind(name)
    .execute(pool)
    .await
    .expect("insert group");
    id
}

fn claims(access_token: &str) -> Value {
    let payload = access_token.split('.').nth(1).expect("a JWT payload");
    let bytes = URL_SAFE_NO_PAD.decode(payload).expect("base64url payload");
    serde_json::from_slice(&bytes).expect("JSON payload")
}

fn names(value: &Value) -> Vec<String> {
    let mut names: Vec<String> = value
        .as_array()
        .map(|roles| {
            roles
                .iter()
                .filter_map(|role| role.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn the_roles_of_a_service_account_reach_its_token() {
    let app = app();

    rt().block_on(async {
        let server = app.server();
        let (client, user_id) = service_account(app).await;
        let suffix = Uuid::new_v4().simple().to_string();
        let realm_role = format!("realm-{suffix}");
        let client_role = format!("client-{suffix}");
        let group_role = format!("group-{suffix}");

        let direct = role(&app.pool, app.realm_id, None, &realm_role).await;
        let scoped = role(&app.pool, app.realm_id, Some(client.id), &client_role).await;
        let inherited = role(&app.pool, app.realm_id, None, &group_role).await;
        assign(&app.pool, user_id, direct).await;
        assign(&app.pool, user_id, scoped).await;

        let organization_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO organizations (id, realm_id, name, alias) VALUES ($1, $2, $3, $3)",
        )
        .bind(organization_id)
        .bind(app.realm_id)
        .bind(format!("org-{suffix}"))
        .execute(&app.pool)
        .await
        .expect("insert organization");
        let parent = group(&app.pool, organization_id, None, "parent").await;
        let child = group(&app.pool, organization_id, Some(parent), "child").await;
        sqlx::query(
            "INSERT INTO organization_group_roles (id, group_id, role_id) VALUES ($1, $2, $3)",
        )
        .bind(Uuid::new_v4())
        .bind(parent)
        .bind(inherited)
        .execute(&app.pool)
        .await
        .expect("attach the role to the parent group");
        sqlx::query(
            "INSERT INTO organization_group_members (id, group_id, user_id) VALUES ($1, $2, $3)",
        )
        .bind(Uuid::new_v4())
        .bind(child)
        .bind(user_id)
        .execute(&app.pool)
        .await
        .expect("add the service account to the child group");

        let response = client_credentials(&server, app, &client).await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        let token = claims(
            response.json::<Value>()["access_token"]
                .as_str()
                .expect("access_token"),
        );

        let mut expected_realm = vec![realm_role, group_role];
        expected_realm.sort();
        assert_eq!(names(&token["realm_access"]["roles"]), expected_realm);
        assert_eq!(
            names(&token["resource_access"][&client.client_id]["roles"]),
            vec![client_role]
        );
    });
}

#[test]
fn a_client_whose_service_account_is_missing_or_disabled_gets_no_token() {
    let app = app();

    rt().block_on(async {
        let server = app.server();
        let unbound = app
            .client(ClientSpec::confidential().with_service_account())
            .await;
        let refused = client_credentials(&server, app, &unbound).await;
        assert!(
            refused.status_code().is_client_error(),
            "an unbound client got {}",
            refused.status_code()
        );

        let (client, user_id) = service_account(app).await;
        sqlx::query("UPDATE users SET enabled = false WHERE id = $1")
            .bind(user_id)
            .execute(&app.pool)
            .await
            .expect("disable the service account");
        let disabled = client_credentials(&server, app, &client).await;
        assert!(
            disabled.status_code().is_client_error(),
            "a disabled service account got {}",
            disabled.status_code()
        );
    });
}

#[test]
fn a_cached_service_account_keeps_its_roles_until_the_ttl_expires() {
    let app = cached_app();

    rt().block_on(async {
        let server = app.server();
        let (client, user_id) = service_account(app).await;
        let before = format!("before-{}", Uuid::new_v4().simple());
        let after = format!("after-{}", Uuid::new_v4().simple());

        let first = role(&app.pool, app.realm_id, None, &before).await;
        assign(&app.pool, user_id, first).await;
        let granted = client_credentials(&server, app, &client).await;
        assert_eq!(granted.status_code(), 200, "{}", granted.text());
        let token = claims(
            granted.json::<Value>()["access_token"]
                .as_str()
                .expect("access_token"),
        );
        assert_eq!(names(&token["realm_access"]["roles"]), vec![before.clone()]);

        let second = role(&app.pool, app.realm_id, None, &after).await;
        assign(&app.pool, user_id, second).await;
        let again = client_credentials(&server, app, &client).await;
        assert_eq!(again.status_code(), 200, "{}", again.text());
        let token = claims(
            again.json::<Value>()["access_token"]
                .as_str()
                .expect("access_token"),
        );
        assert_eq!(names(&token["realm_access"]["roles"]), vec![before]);
    });
}

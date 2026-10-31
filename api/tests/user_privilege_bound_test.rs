//! An administrator holding `manage_users` must not take over an account that
//! holds more than it does. Resetting a password, removing a credential or
//! rewriting the profile of a `manage_realm` account hands over the realm, so
//! each of them is refused, exactly as `assign_role` refuses to grant a role
//! the subject does not hold.

mod common;

use axum_test::{TestResponse, TestServer};
use common::{ADMIN_CLIENT_ID, SeededUser, TestApp, app, password_grant, rt};
use serde_json::{Value, json};
use uuid::Uuid;

const PASSWORD: &str = "Privilege-B0und-Passw0rd!";
const STOLEN_PASSWORD: &str = "Taken-0ver-Passw0rd!";

struct Fixture {
    attacker_token: String,
    victim: SeededUser,
    bystander: SeededUser,
}

async fn create_role(
    server: &TestServer,
    app: &TestApp,
    token: &str,
    permissions: &[&str],
) -> String {
    let response = server
        .post(&app.path("/roles"))
        .authorization_bearer(token)
        .json(&json!({
            "name": format!("role-{}", Uuid::new_v4().simple()),
            "description": "privilege bound test",
            "permissions": permissions,
        }))
        .await;
    assert_eq!(
        response.status_code(),
        201,
        "create role: {}",
        response.text()
    );
    response.json::<Value>()["data"]["id"]
        .as_str()
        .expect("created role id")
        .to_string()
}

async fn assign_role(
    server: &TestServer,
    app: &TestApp,
    token: &str,
    user_id: &str,
    role_id: &str,
) {
    let response = server
        .post(&app.path(&format!("/users/{user_id}/roles/{role_id}")))
        .authorization_bearer(token)
        .json(&json!({}))
        .await;
    assert_eq!(
        response.status_code(),
        200,
        "assign role: {}",
        response.text()
    );
}

async fn access_token(server: &TestServer, app: &TestApp, user: &SeededUser) -> String {
    password_grant(
        server,
        app,
        ADMIN_CLIENT_ID,
        None,
        &user.username,
        &user.password,
    )
    .await
    .json::<Value>()["access_token"]
        .as_str()
        .expect("access_token")
        .to_string()
}

async fn fixture(server: &TestServer, app: &TestApp) -> Fixture {
    let admin_token = app.admin_token(server).await;

    let attacker = app.user(server, PASSWORD).await;
    let user_admin = create_role(server, app, &admin_token, &["manage_users", "view_roles"]).await;
    assign_role(server, app, &admin_token, &attacker.id, &user_admin).await;

    let victim = app.user(server, PASSWORD).await;
    let realm_admin = create_role(server, app, &admin_token, &["manage_realm"]).await;
    assign_role(server, app, &admin_token, &victim.id, &realm_admin).await;

    let bystander = app.user(server, PASSWORD).await;

    Fixture {
        attacker_token: access_token(server, app, &attacker).await,
        victim,
        bystander,
    }
}

async fn reset_password(
    server: &TestServer,
    app: &TestApp,
    token: &str,
    user_id: &str,
) -> TestResponse {
    server
        .put(&app.path(&format!("/users/{user_id}/reset-password")))
        .authorization_bearer(token)
        .json(&json!({
            "value": STOLEN_PASSWORD,
            "temporary": false,
            "credential_type": "password",
        }))
        .await
}

async fn signs_in(server: &TestServer, app: &TestApp, user: &SeededUser) -> bool {
    password_grant(
        server,
        app,
        ADMIN_CLIENT_ID,
        None,
        &user.username,
        &user.password,
    )
    .await
    .status_code()
        == 200
}

async fn password_credential(app: &TestApp, user_id: &str) -> Uuid {
    sqlx::query_scalar(
        "SELECT id FROM credentials WHERE user_id = $1::uuid AND credential_type = 'password'",
    )
    .bind(user_id)
    .fetch_one(&app.pool)
    .await
    .expect("password credential")
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test user_privilege_bound_test -- --ignored"]
fn a_user_administrator_cannot_reset_the_password_of_a_realm_administrator() {
    let app = app();

    rt().block_on(async {
        let server = app.server();
        let f = fixture(&server, app).await;

        let response = reset_password(&server, app, &f.attacker_token, &f.victim.id).await;

        assert_eq!(response.status_code(), 403, "{}", response.text());
        assert!(signs_in(&server, app, &f.victim).await);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test user_privilege_bound_test -- --ignored"]
fn a_user_administrator_cannot_delete_a_credential_of_a_realm_administrator() {
    let app = app();

    rt().block_on(async {
        let server = app.server();
        let f = fixture(&server, app).await;
        let credential_id = password_credential(app, &f.victim.id).await;

        let response = server
            .delete(&app.path(&format!(
                "/users/{}/credentials/{credential_id}",
                f.victim.id
            )))
            .authorization_bearer(&f.attacker_token)
            .await;

        assert_eq!(response.status_code(), 403, "{}", response.text());
        assert!(signs_in(&server, app, &f.victim).await);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test user_privilege_bound_test -- --ignored"]
fn a_user_administrator_cannot_rewrite_the_email_of_a_realm_administrator() {
    let app = app();

    rt().block_on(async {
        let server = app.server();
        let f = fixture(&server, app).await;

        let response = server
            .put(&app.path(&format!("/users/{}", f.victim.id)))
            .authorization_bearer(&f.attacker_token)
            .json(&json!({
                "firstname": "Taken",
                "lastname": "Over",
                "email": "attacker@ferriskey.test",
                "email_verified": true,
                "enabled": true,
            }))
            .await;

        assert_eq!(response.status_code(), 403, "{}", response.text());
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test user_privilege_bound_test -- --ignored"]
fn a_user_administrator_cannot_delete_a_realm_administrator() {
    let app = app();

    rt().block_on(async {
        let server = app.server();
        let f = fixture(&server, app).await;

        let response = server
            .delete(&app.path(&format!("/users/{}", f.victim.id)))
            .authorization_bearer(&f.attacker_token)
            .await;

        assert_eq!(response.status_code(), 403, "{}", response.text());
        assert!(signs_in(&server, app, &f.victim).await);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test user_privilege_bound_test -- --ignored"]
fn a_user_administrator_still_resets_the_password_of_a_plain_user() {
    let app = app();

    rt().block_on(async {
        let server = app.server();
        let f = fixture(&server, app).await;

        let response = reset_password(&server, app, &f.attacker_token, &f.bystander.id).await;

        assert_eq!(response.status_code(), 200, "{}", response.text());
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test user_privilege_bound_test -- --ignored"]
fn a_realm_administrator_still_resets_the_password_of_a_realm_administrator() {
    let app = app();

    rt().block_on(async {
        let server = app.server();
        let f = fixture(&server, app).await;
        let admin_token = app.admin_token(&server).await;

        let response = reset_password(&server, app, &admin_token, &f.victim.id).await;

        assert_eq!(response.status_code(), 200, "{}", response.text());
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test user_privilege_bound_test -- --ignored"]
fn a_bulk_delete_reaching_a_realm_administrator_deletes_nobody() {
    let app = app();

    rt().block_on(async {
        let server = app.server();
        let f = fixture(&server, app).await;

        let response = server
            .delete(&app.path("/users/bulk"))
            .authorization_bearer(&f.attacker_token)
            .json(&json!({ "ids": [f.bystander.id, f.victim.id] }))
            .await;

        assert_eq!(response.status_code(), 403, "{}", response.text());
        assert!(signs_in(&server, app, &f.victim).await);
        assert!(signs_in(&server, app, &f.bystander).await);
    });
}

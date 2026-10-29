mod common;

use axum_test::{TestResponse, TestServer};
use common::{
    ClientSpec, SeededClient, SeededUser, TestApp, decode_unverified, introspect,
    jwt_view_of_opaque_app, opaque_app, password_grant, refresh, rt,
};
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::task::JoinSet;

const PASSWORD: &str = "Correct-horse-battery-7";

struct Login {
    client: SeededClient,
    user: SeededUser,
    tokens: Value,
}

impl Login {
    fn refresh_token(&self) -> &str {
        self.tokens["refresh_token"]
            .as_str()
            .expect("refresh token")
    }

    fn access_token(&self) -> &str {
        self.tokens["access_token"].as_str().expect("access token")
    }
}

async fn login_with(app: &TestApp, server: &TestServer, spec: ClientSpec) -> Login {
    let client = app.client(spec).await;
    let user = app.user(server, PASSWORD).await;
    let granted = password_grant(
        server,
        app,
        &client.client_id,
        client.secret.as_deref(),
        &user.username,
        &user.password,
    )
    .await;
    assert_eq!(granted.status_code(), 200, "{}", granted.text());
    Login {
        client,
        user,
        tokens: granted.json(),
    }
}

async fn login(app: &TestApp, server: &TestServer) -> Login {
    login_with(app, server, ClientSpec::public().with_direct_access()).await
}

async fn refresh_as(
    app: &TestApp,
    server: &TestServer,
    client: &SeededClient,
    token: &str,
) -> TestResponse {
    refresh(
        server,
        app,
        &client.client_id,
        client.secret.as_deref(),
        token,
    )
    .await
}

fn refresh_token_of(response: &TestResponse) -> String {
    response.json::<Value>()["refresh_token"]
        .as_str()
        .expect("refresh token")
        .to_string()
}

fn is_opaque(token: &str) -> bool {
    Regex::new(r"^fkr_[A-Za-z0-9_-]{43}$")
        .expect("regex")
        .is_match(token)
}

fn sha256(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

fn assert_invalid_grant(response: &TestResponse) {
    assert_eq!(response.status_code(), 400, "{}", response.text());
    assert_eq!(
        response.json::<Value>()["error"],
        "invalid_grant",
        "{}",
        response.text()
    );
}

fn stable_claims(token: &str) -> Value {
    let mut claims = decode_unverified(token);
    let object = claims.as_object_mut().expect("claims object");
    for volatile in [
        "iat",
        "exp",
        "jti",
        "sid",
        "auth_time",
        "session_state",
        "at_hash",
    ] {
        object.remove(volatile);
    }
    claims
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_1_the_password_grant_returns_an_opaque_token_and_stores_only_its_hash() {
    let app = opaque_app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;
        let token = login.refresh_token();

        assert!(is_opaque(token), "{token}");
        let by_hash: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens WHERE token_hash = $1")
                .bind(sha256(token))
                .fetch_one(&app.pool)
                .await
                .expect("count by hash");
        let verbatim: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM refresh_tokens WHERE refresh_tokens::text LIKE '%' || $1 || '%'",
        )
        .bind(&token[4..])
        .fetch_one(&app.pool)
        .await
        .expect("search for the secret");
        assert_eq!(by_hash, 1);
        assert_eq!(verbatim, 0);
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_2_an_opaque_refresh_returns_an_opaque_token_with_the_claims_of_the_jwt_format() {
    let opaque = opaque_app();
    let jwt = jwt_view_of_opaque_app();
    rt().block_on(async {
        let server = opaque.server();
        let jwt_server = jwt.server();
        let login = login(opaque, &server).await;
        let user = &login.user;

        let refreshed = refresh_as(opaque, &server, &login.client, login.refresh_token()).await;
        assert_eq!(refreshed.status_code(), 200, "{}", refreshed.text());
        let next = refresh_token_of(&refreshed);
        assert!(is_opaque(&next), "{next}");
        assert_ne!(next, login.refresh_token());

        let via_jwt = password_grant(
            &jwt_server,
            jwt,
            &login.client.client_id,
            None,
            &user.username,
            &user.password,
        )
        .await;
        let jwt_refresh_token = refresh_token_of(&via_jwt);
        assert_eq!(jwt_refresh_token.matches('.').count(), 2);
        let jwt_refreshed = refresh_as(jwt, &jwt_server, &login.client, &jwt_refresh_token).await;
        assert_eq!(jwt_refreshed.status_code(), 200, "{}", jwt_refreshed.text());

        let body: Value = refreshed.json();
        let jwt_body: Value = jwt_refreshed.json();
        let opaque_claims = decode_unverified(body["access_token"].as_str().expect("access"));
        assert_eq!(
            opaque_claims["sid"],
            decode_unverified(login.access_token())["sid"]
        );
        assert!(opaque_claims["sid"].is_string(), "{opaque_claims}");
        assert_eq!(
            stable_claims(body["access_token"].as_str().expect("access")),
            stable_claims(jwt_body["access_token"].as_str().expect("access")),
        );
        assert_eq!(
            stable_claims(body["id_token"].as_str().expect("id token")),
            stable_claims(jwt_body["id_token"].as_str().expect("id token")),
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_3_reused_opaque_token_revokes_family() {
    let app = opaque_app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;

        let first = refresh_as(app, &server, &login.client, login.refresh_token()).await;
        assert_eq!(first.status_code(), 200, "{}", first.text());
        let successor = refresh_token_of(&first);

        let replay = refresh_as(app, &server, &login.client, login.refresh_token()).await;
        assert_invalid_grant(&replay);

        let successor_try = refresh_as(app, &server, &login.client, &successor).await;
        assert_invalid_grant(&successor_try);
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_4_concurrent_refreshes_of_one_opaque_token_yield_exactly_one_success() {
    let app = opaque_app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;

        let mut attempts = JoinSet::new();
        for _ in 0..8 {
            let server = app.server();
            let client_id = login.client.client_id.clone();
            let token = login.refresh_token().to_string();
            attempts.spawn(async move {
                refresh(&server, app, &client_id, None, &token)
                    .await
                    .status_code()
                    .as_u16()
            });
        }
        let mut statuses = Vec::new();
        while let Some(status) = attempts.join_next().await {
            statuses.push(status.expect("join"));
        }

        assert_eq!(
            statuses.iter().filter(|status| **status == 200).count(),
            1,
            "{statuses:?}"
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_5_a_token_presented_by_another_client_is_refused_and_stays_usable() {
    let app = opaque_app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;
        let other = app.client(ClientSpec::public().with_direct_access()).await;

        let stolen = refresh_as(app, &server, &other, login.refresh_token()).await;
        assert_invalid_grant(&stolen);

        let own = refresh_as(app, &server, &login.client, login.refresh_token()).await;
        assert_eq!(own.status_code(), 200, "{}", own.text());
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_6_an_expired_opaque_token_is_refused() {
    let app = opaque_app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;
        sqlx::query(
            "UPDATE refresh_tokens SET expires_at = (now() AT TIME ZONE 'utc') - interval '1 minute' \
             WHERE token_hash = $1",
        )
        .bind(sha256(login.refresh_token()))
        .execute(&app.pool)
        .await
        .expect("expire the token");

        let response = refresh_as(app, &server, &login.client, login.refresh_token()).await;

        assert_invalid_grant(&response);
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_7_an_opaque_token_of_a_revoked_session_is_refused() {
    let app = opaque_app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;
        let expired = sqlx::query(
            "UPDATE user_sessions SET expires_at = (now() AT TIME ZONE 'utc') - interval '1 minute' \
             WHERE id = (SELECT session_id FROM refresh_tokens WHERE token_hash = $1)",
        )
        .bind(sha256(login.refresh_token()))
        .execute(&app.pool)
        .await
        .expect("expire the session");
        assert_eq!(expired.rows_affected(), 1);

        let response = refresh_as(app, &server, &login.client, login.refresh_token()).await;

        assert_invalid_grant(&response);
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_8_a_legacy_jwt_is_accepted_once_and_answered_with_an_opaque_token() {
    let opaque = opaque_app();
    let jwt = jwt_view_of_opaque_app();
    rt().block_on(async {
        let server = opaque.server();
        let jwt_server = jwt.server();
        let client = opaque
            .client(ClientSpec::public().with_direct_access())
            .await;
        let user = opaque.user(&server, PASSWORD).await;
        let issued = password_grant(
            &jwt_server,
            jwt,
            &client.client_id,
            None,
            &user.username,
            &user.password,
        )
        .await;
        let legacy = refresh_token_of(&issued);
        assert_eq!(legacy.matches('.').count(), 2, "{legacy}");

        let response = refresh_as(opaque, &server, &client, &legacy).await;

        assert_eq!(response.status_code(), 200, "{}", response.text());
        assert!(is_opaque(&refresh_token_of(&response)));
        let replay = refresh_as(opaque, &server, &client, &legacy).await;
        assert_invalid_grant(&replay);
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_9_an_opaque_token_is_accepted_by_the_jwt_format_and_answered_with_a_jwt() {
    let opaque = opaque_app();
    let jwt = jwt_view_of_opaque_app();
    rt().block_on(async {
        let server = opaque.server();
        let jwt_server = jwt.server();
        let login = login(opaque, &server).await;
        assert!(is_opaque(login.refresh_token()));

        let response = refresh_as(jwt, &jwt_server, &login.client, login.refresh_token()).await;

        assert_eq!(response.status_code(), 200, "{}", response.text());
        let next = refresh_token_of(&response);
        assert_eq!(next.matches('.').count(), 2, "{next}");
        let again = refresh_as(jwt, &jwt_server, &login.client, &next).await;
        assert_eq!(again.status_code(), 200, "{}", again.text());
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_10_introspection_reports_an_active_opaque_token_until_it_rotates() {
    let app = opaque_app();
    rt().block_on(async {
        let server = app.server();
        let login = login_with(
            app,
            &server,
            ClientSpec::confidential().with_direct_access(),
        )
        .await;

        let active = introspect(&server, app, &login.client, login.refresh_token()).await;

        assert_eq!(active["active"], true, "{active}");
        assert_eq!(active["sub"], login.user.id, "{active}");
        assert_eq!(active["client_id"], login.client.client_id, "{active}");
        assert_eq!(
            active["scope"],
            decode_unverified(login.access_token())["scope"],
            "{active}"
        );
        assert_eq!(active["token_type"], "Refresh", "{active}");
        assert!(active["exp"].is_i64(), "{active}");
        assert!(active["iat"].is_i64(), "{active}");

        let refreshed = refresh_as(app, &server, &login.client, login.refresh_token()).await;
        assert_eq!(refreshed.status_code(), 200, "{}", refreshed.text());

        let rotated = introspect(&server, app, &login.client, login.refresh_token()).await;
        assert_eq!(rotated["active"], false, "{rotated}");
    });
}

async fn revoke(
    app: &TestApp,
    server: &TestServer,
    client: &SeededClient,
    token: &str,
) -> TestResponse {
    server
        .post(&app.oidc("revoke"))
        .form(&[("client_id", client.client_id.as_str()), ("token", token)])
        .await
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_11_revocation_by_the_owning_client_stops_refreshes_and_by_another_is_a_no_op() {
    let app = opaque_app();
    rt().block_on(async {
        let server = app.server();
        let kept = login(app, &server).await;
        let other = app.client(ClientSpec::public().with_direct_access()).await;

        let foreign = revoke(app, &server, &other, kept.refresh_token()).await;
        assert_eq!(foreign.status_code(), 200, "{}", foreign.text());

        let revoked_by_owner = {
            let login = login_with(app, &server, ClientSpec::public().with_direct_access()).await;
            let revoked = revoke(app, &server, &login.client, login.refresh_token()).await;
            assert_eq!(revoked.status_code(), 200, "{}", revoked.text());
            refresh_as(app, &server, &login.client, login.refresh_token()).await
        };
        assert_invalid_grant(&revoked_by_owner);

        let still_usable = refresh_as(app, &server, &kept.client, kept.refresh_token()).await;
        assert_eq!(still_usable.status_code(), 200, "{}", still_usable.text());
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_12_a_malformed_token_is_an_invalid_grant() {
    let app = opaque_app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;
        let wrong_length = format!("fkr_{}", "A".repeat(42));

        for malformed in [wrong_length.as_str(), "fkr_", "not-a-token"] {
            let response = refresh_as(app, &server, &login.client, malformed).await;
            assert_invalid_grant(&response);
        }
    });
}

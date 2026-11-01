//! Moves a realm between two instances, each on its own schema and with its
//! own SECRETS_ENCRYPTION_KEYS, and checks that what users and relying parties
//! hold keeps working on the target (#1609).
//!
//!   cargo test -p ferriskey-api --test realm_transfer_test -- --ignored

mod common;

use axum::body::Bytes;
use axum::http::HeaderValue;
use axum_test::TestServer;
use base64::prelude::{BASE64_URL_SAFE_NO_PAD, Engine as _};
use chrono::Utc;
use common::{
    CALLBACK, ClientSpec, SeededClient, TestApp, authorize, introspect, master_instance,
    password_grant, query_param, refresh, rt,
};
use ferriskey_api::args::Args;
use ferriskey_core::domain::credential::entities::CredentialData;
use ferriskey_core::domain::realm_transfer::bundle::{BundleHeader, BundleRealm, BundleRow};
use ferriskey_core::domain::realm_transfer::codec::BundleWriter;
use ferriskey_core::domain::realm_transfer::ports::RowSink;
use ferriskey_security::crypto::bundle_envelope::{BundleKdf, BundleKey};
use p256::ecdsa::{Signature, SigningKey, signature::Signer};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use uuid::Uuid;
use webauthn_rs::prelude::{
    AttestationFormat, COSEAlgorithm, COSEEC2Key, COSEKey, COSEKeyType,
    Credential as WebAuthnCredential, ECDSACurve, ParsedAttestation,
};

const SOURCE_RING: &str = "source-ring:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";
const TARGET_RING: &str = "target-ring:ICEiIyQlJicoKSorLC0uLzAxMjM0NTY3ODk6Ozw9Pj8=";
const PASSPHRASE: &str = "a long passphrase for the move";
const PASSPHRASE_HEADER: &str = "x-ferriskey-bundle-passphrase";
const USER_PASSWORD: &str = "Moved-Across-Instances-1!";
const PASSKEY_ID: &[u8] = b"realm-transfer-passkey";

struct Realm {
    name: String,
    id: Uuid,
}

async fn admin_token(instance: &TestApp, server: &TestServer) -> String {
    instance.admin_token(server).await
}

async fn create_realm(instance: &TestApp, server: &TestServer) -> Realm {
    let token = admin_token(instance, server).await;
    let name = format!("acme-{}", Uuid::new_v4().simple());
    let created = server
        .post("/realms")
        .authorization_bearer(&token)
        .json(&json!({ "name": name }))
        .await;
    assert_eq!(
        created.status_code(),
        201,
        "create realm: {}",
        created.text()
    );

    let id = sqlx::query_scalar("SELECT id FROM realms WHERE name = $1")
        .bind(&name)
        .fetch_one(&instance.pool)
        .await
        .expect("realm id");
    sqlx::query("UPDATE realm_settings SET passkey_enabled = true WHERE realm_id = $1")
        .bind(id)
        .execute(&instance.pool)
        .await
        .expect("enable passkeys");

    Realm { name, id }
}

async fn create_user(instance: &TestApp, server: &TestServer, realm: &Realm) -> (Uuid, String) {
    let token = admin_token(instance, server).await;
    let username = format!("mover-{}", Uuid::new_v4().simple());
    let created = server
        .post(&format!("/realms/{}/users", realm.name))
        .authorization_bearer(&token)
        .json(&json!({
            "username": username,
            "firstname": "Moving",
            "lastname": "User",
            "email": format!("{username}@acme.test"),
            "email_verified": true,
        }))
        .await;
    assert_eq!(
        created.status_code(),
        200,
        "create user: {}",
        created.text()
    );
    let id: Uuid = created.json::<Value>()["data"]["id"]
        .as_str()
        .and_then(|id| id.parse().ok())
        .expect("user id");

    let reset = server
        .put(&format!("/realms/{}/users/{id}/reset-password", realm.name))
        .authorization_bearer(&token)
        .json(&json!({ "value": USER_PASSWORD, "temporary": false, "credential_type": "password" }))
        .await;
    assert_eq!(reset.status_code(), 200, "reset password: {}", reset.text());

    (id, username)
}

fn passkey_key() -> SigningKey {
    SigningKey::from_slice(&[0x24; 32]).expect("a valid p256 scalar")
}

async fn enrol_passkey(instance: &TestApp, user_id: Uuid) {
    let point = passkey_key().verifying_key().to_encoded_point(false);
    let credential = WebAuthnCredential {
        cred_id: PASSKEY_ID.to_vec().into(),
        cred: COSEKey {
            type_: COSEAlgorithm::ES256,
            key: COSEKeyType::EC_EC2(COSEEC2Key {
                curve: ECDSACurve::SECP256R1,
                x: point.x().expect("x").to_vec().into(),
                y: point.y().expect("y").to_vec().into(),
            }),
        },
        counter: 0,
        transports: None,
        user_verified: true,
        backup_eligible: false,
        backup_state: false,
        registration_policy: Default::default(),
        extensions: Default::default(),
        attestation: ParsedAttestation::default(),
        attestation_format: AttestationFormat::None,
    };
    let data = serde_json::to_value(CredentialData::WebAuthn {
        credential: Box::new(credential),
    })
    .expect("passkey data");

    sqlx::query(
        "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data, webauthn_credential_id) \
         VALUES ($1, 'webauthn-public-key-credential', $2, '', $3::jsonb, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(user_id)
    .bind(data)
    .bind(PASSKEY_ID)
    .execute(&instance.pool)
    .await
    .expect("enrol passkey");
}

async fn sso_cookie(
    server: &TestServer,
    view: &TestApp,
    client: &SeededClient,
    username: &str,
) -> String {
    let authorized = authorize(server, view, &client.client_id, &[]).await;
    assert_eq!(
        authorized.status_code(),
        302,
        "authorize: {}",
        authorized.text()
    );

    let login = server
        .post(&view.path("/login-actions/authenticate"))
        .add_cookie(authorized.cookie("FERRISKEY_SESSION"))
        .add_query_param("client_id", &client.client_id)
        .json(&json!({ "username": username, "password": USER_PASSWORD }))
        .await;
    assert_eq!(login.status_code(), 200, "login: {}", login.text());

    login.cookie("FERRISKEY_SSO").value().to_string()
}

async fn export(server: &TestServer, token: &str, realm: &Realm, passphrase: &str) -> Bytes {
    let response = server
        .post(&format!("/realms/{}/export", realm.name))
        .authorization_bearer(token)
        .add_header(
            PASSPHRASE_HEADER,
            HeaderValue::from_str(passphrase).unwrap(),
        )
        .await;
    assert_eq!(response.status_code(), 200, "export: {}", response.text());
    assert_eq!(
        response.headers()["content-type"],
        "application/vnd.ferriskey.realm-bundle"
    );
    response.into_bytes()
}

async fn import(
    server: &TestServer,
    token: &str,
    bundle: Bytes,
    passphrase: &str,
) -> axum_test::TestResponse {
    server
        .post("/realms/import")
        .authorization_bearer(token)
        .add_header(
            PASSPHRASE_HEADER,
            HeaderValue::from_str(passphrase).unwrap(),
        )
        .content_type("application/vnd.ferriskey.realm-bundle")
        .bytes(bundle)
        .await
}

fn reason(response: &axum_test::TestResponse) -> String {
    response.json::<Value>()["reason"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

async fn row_counts(instance: &TestApp, realm_id: Uuid) -> Vec<i64> {
    let mut counts = Vec::new();
    for sql in [
        "SELECT COUNT(*) FROM users WHERE realm_id = $1",
        "SELECT COUNT(*) FROM clients WHERE realm_id = $1",
        "SELECT COUNT(*) FROM roles WHERE realm_id = $1",
        "SELECT COUNT(*) FROM jwt_keys WHERE realm_id = $1",
        "SELECT COUNT(*) FROM user_sessions WHERE realm_id = $1",
        "SELECT COUNT(*) FROM credentials c JOIN users u ON u.id = c.user_id WHERE u.realm_id = $1",
        "SELECT COUNT(*) FROM user_role ur JOIN users u ON u.id = ur.user_id WHERE u.realm_id = $1",
        "SELECT COUNT(*) FROM refresh_tokens r JOIN users u ON u.id = r.user_id WHERE u.realm_id = $1",
    ] {
        counts.push(
            sqlx::query_scalar(sql)
                .bind(realm_id)
                .fetch_one(&instance.pool)
                .await
                .expect("count"),
        );
    }
    counts
}

async fn passkey_login(server: &TestServer, view: &TestApp, client: &SeededClient, user_id: Uuid) {
    let authorized = authorize(server, view, &client.client_id, &[]).await;
    assert_eq!(
        authorized.status_code(),
        302,
        "authorize: {}",
        authorized.text()
    );
    let session = authorized.cookie("FERRISKEY_SESSION");

    let options = server
        .post(&view.path("/login-actions/passkey-request-options"))
        .add_cookie(session.clone())
        .json(&json!({ "username": null }))
        .await;
    assert_eq!(
        options.status_code(),
        200,
        "passkey options: {}",
        options.text()
    );
    let challenge = options.json::<Value>()["publicKey"]["challenge"]
        .as_str()
        .expect("a challenge")
        .to_string();

    let origin = Args::default().webapp_url.trim_end_matches('/').to_string();
    let rp_id = url::Url::parse(&origin)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .unwrap_or_else(|| "localhost".to_string());
    let client_data = json!({
        "type": "webauthn.get",
        "challenge": challenge,
        "origin": origin,
        "crossOrigin": false,
    })
    .to_string();
    let mut authenticator_data = Sha256::digest(rp_id.as_bytes()).to_vec();
    authenticator_data.push(0x05);
    authenticator_data.extend_from_slice(&1u32.to_be_bytes());
    let mut signed = authenticator_data.clone();
    signed.extend_from_slice(&Sha256::digest(client_data.as_bytes()));
    let signature: Signature = passkey_key().sign(&signed);

    let authenticated = server
        .post(&view.path("/login-actions/passkey-authenticate"))
        .add_cookie(session)
        .json(&json!({
            "id": BASE64_URL_SAFE_NO_PAD.encode(PASSKEY_ID),
            "rawId": BASE64_URL_SAFE_NO_PAD.encode(PASSKEY_ID),
            "response": {
                "authenticatorData": BASE64_URL_SAFE_NO_PAD.encode(&authenticator_data),
                "clientDataJSON": BASE64_URL_SAFE_NO_PAD.encode(client_data.as_bytes()),
                "signature": BASE64_URL_SAFE_NO_PAD.encode(signature.to_der().as_bytes()),
                "userHandle": BASE64_URL_SAFE_NO_PAD.encode(user_id.as_bytes()),
            },
            "type": "public-key",
        }))
        .await;
    assert_eq!(
        authenticated.status_code(),
        200,
        "passkey login: {}",
        authenticated.text()
    );
    let login_url = authenticated.json::<Value>()["login_url"]
        .as_str()
        .expect("login url")
        .to_string();
    assert!(query_param(&login_url, "code").is_some(), "{login_url}");
}

async fn keyless_bundle(passphrase: &str) -> Bytes {
    let realm = BundleRealm {
        id: Uuid::now_v7(),
        name: format!("keyless-{}", Uuid::new_v4().simple()),
    };
    let key = BundleKey::create(
        passphrase,
        BundleKdf {
            memory_kib: 64,
            passes: 1,
            lanes: 1,
        },
    )
    .expect("bundle key");
    let (out, mut chunks) = mpsc::channel(64);
    let mut writer = BundleWriter::start(key, BundleHeader::new(realm.clone(), 0, Utc::now()), out)
        .await
        .expect("writer");
    let now = Utc::now().naive_utc();
    writer
        .accept(BundleRow {
            table: "realms".to_string(),
            row:
                json!({ "id": realm.id, "name": realm.name, "created_at": now, "updated_at": now })
                    .as_object()
                    .cloned()
                    .expect("realm row"),
        })
        .await
        .expect("realm row");
    writer.finish().await.expect("finish");

    let mut bundle = Vec::new();
    while let Some(chunk) = chunks.recv().await {
        bundle.extend(chunk.expect("chunk"));
    }
    Bytes::from(bundle)
}

#[test]
#[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_transfer_test -- --ignored"]
fn a_realm_moved_to_another_instance_keeps_tokens_sessions_and_logins_working() {
    rt().block_on(async {
        let source = master_instance(SOURCE_RING).await;
        let target = master_instance(TARGET_RING).await;
        let source_server = source.server();
        let target_server = target.server();

        let realm = create_realm(&source, &source_server).await;
        let on_source = source.realm_view(&realm.name, realm.id);
        let client = on_source
            .client_in(realm.id, ClientSpec::confidential().with_direct_access())
            .await;
        let (user_id, username) = create_user(&source, &source_server, &realm).await;
        enrol_passkey(&source, user_id).await;

        let tokens = password_grant(
            &source_server,
            &on_source,
            &client.client_id,
            client.secret.as_deref(),
            &username,
            USER_PASSWORD,
        )
        .await;
        assert_eq!(tokens.status_code(), 200, "password grant: {}", tokens.text());
        let tokens: Value = tokens.json();
        let access_token = tokens["access_token"].as_str().expect("access token").to_string();
        let refresh_token = tokens["refresh_token"].as_str().expect("refresh token").to_string();
        let sso = sso_cookie(&source_server, &on_source, &client, &username).await;

        let source_admin = admin_token(&source, &source_server).await;
        let bundle = export(&source_server, &source_admin, &realm, PASSPHRASE).await;
        for clear in [
            b"PRIVATE KEY".as_slice(),
            b"$argon2".as_slice(),
            username.as_bytes(),
            client.secret.as_deref().unwrap_or_default().as_bytes(),
        ] {
            assert!(!contains(&bundle, clear), "the bundle shows {:?}", String::from_utf8_lossy(clear));
        }

        let target_admin = admin_token(&target, &target_server).await;
        let refused = import(&target_server, &target_admin, bundle.clone(), "not the passphrase given").await;
        assert_eq!(refused.status_code(), 400, "{}", refused.text());
        let cut = import(&target_server, &target_admin, bundle.slice(..bundle.len() / 2), PASSPHRASE).await;
        assert_eq!(cut.status_code(), 400, "{}", cut.text());
        assert_eq!(row_counts(&target, realm.id).await, vec![0; 8], "a refused import leaves nothing");

        let imported = import(&target_server, &target_admin, bundle.clone(), PASSPHRASE).await;
        assert_eq!(imported.status_code(), 201, "import: {}", imported.text());
        assert_eq!(imported.json::<Value>()["data"]["realm_id"], realm.id.to_string());
        let moved = row_counts(&target, realm.id).await;
        assert_eq!(moved, row_counts(&source, realm.id).await);

        let again = import(&target_server, &target_admin, bundle, PASSPHRASE).await;
        assert_eq!(again.status_code(), 200, "re-import: {}", again.text());
        assert_eq!(row_counts(&target, realm.id).await, moved, "a re-import changes nothing");

        let sealed: String = sqlx::query_scalar(
            "SELECT private_key FROM jwt_keys WHERE realm_id = $1 AND rotated_at IS NULL AND revoked_at IS NULL",
        )
        .bind(realm.id)
        .fetch_one(&target.pool)
        .await
        .expect("active key");
        assert!(sealed.starts_with("fk1:target-ring:"), "the key is sealed for the target");

        let on_target = target.realm_view(&realm.name, realm.id);

        let introspection = introspect(&target_server, &on_target, &client, &access_token).await;
        assert_eq!(introspection["active"], true, "{introspection}");
        assert_eq!(introspection["sub"], user_id.to_string());

        let refreshed = refresh(
            &target_server,
            &on_target,
            &client.client_id,
            client.secret.as_deref(),
            &refresh_token,
        )
        .await;
        assert_eq!(refreshed.status_code(), 200, "refresh: {}", refreshed.text());

        let skipped = target_server
            .get(&on_target.oidc("auth"))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", &client.client_id)
            .add_query_param("redirect_uri", CALLBACK)
            .add_query_param("scope", "openid")
            .add_header("Cookie", HeaderValue::from_str(&format!("FERRISKEY_SSO={sso}")).unwrap())
            .await;
        let location = skipped
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();
        assert!(
            location.starts_with(CALLBACK) && query_param(&location, "code").is_some(),
            "the SSO cookie must skip the login page, got {} to {location}",
            skipped.status_code()
        );

        let password = password_grant(
            &target_server,
            &on_target,
            &client.client_id,
            client.secret.as_deref(),
            &username,
            USER_PASSWORD,
        )
        .await;
        assert_eq!(password.status_code(), 200, "password login: {}", password.text());

        passkey_login(&target_server, &on_target, &client, user_id).await;

        let event: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM security_events WHERE realm_id = $1 AND event_type = 'realm_imported'",
        )
        .bind(realm.id)
        .fetch_one(&target.pool)
        .await
        .expect("events");
        assert_eq!(event, 2);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_transfer_test -- --ignored"]
fn a_bundle_without_signing_keys_is_refused_and_leaves_nothing() {
    rt().block_on(async {
        let target = master_instance(TARGET_RING).await;
        let server = target.server();
        let token = admin_token(&target, &server).await;

        let refused = import(
            &server,
            &token,
            keyless_bundle(PASSPHRASE).await,
            PASSPHRASE,
        )
        .await;

        assert_eq!(refused.status_code(), 400, "{}", refused.text());
        assert_eq!(reason(&refused), "realm_bundle_without_keys");
        let realms: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM realms WHERE name LIKE 'keyless-%'")
                .fetch_one(&target.pool)
                .await
                .expect("count");
        assert_eq!(realms, 0);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_transfer_test -- --ignored"]
fn the_master_realm_cannot_be_exported_and_a_short_passphrase_is_refused() {
    rt().block_on(async {
        let source = master_instance("").await;
        let server = source.server();
        let token = admin_token(&source, &server).await;

        let master = server
            .post("/realms/master/export")
            .authorization_bearer(&token)
            .add_header(PASSPHRASE_HEADER, HeaderValue::from_static(PASSPHRASE))
            .await;
        assert_eq!(master.status_code(), 400, "{}", master.text());
        assert_eq!(reason(&master), "master_realm_not_transferable");

        let realm = create_realm(&source, &server).await;
        let short = server
            .post(&format!("/realms/{}/export", realm.name))
            .authorization_bearer(&token)
            .add_header(PASSPHRASE_HEADER, HeaderValue::from_static("short"))
            .await;
        assert_eq!(short.status_code(), 400, "{}", short.text());
        assert_eq!(reason(&short), "invalid_bundle_passphrase");

        let anonymous = server
            .post(&format!("/realms/{}/export", realm.name))
            .add_header(PASSPHRASE_HEADER, HeaderValue::from_static(PASSPHRASE))
            .await;
        assert_eq!(anonymous.status_code(), 401, "{}", anonymous.text());
    });
}

//! The behavioural oracle: every `AUTHZ_MATRIX` row, exercised against the
//! engine that ships today.
//!
//! `matrix_coverage` proves the matrix *describes* the source. This file proves
//! it *predicts* the engine's answers — which is what makes it usable as a
//! reference when a second engine is introduced.
//!
//! Every `can_*` body in the codebase has the same shape: resolve the subject,
//! resolve its permissions for the target realm, then test them against a fixed
//! slice. So the decision is fully determined by
//! `get_permission_for_target_realm` plus the row's slice, and that is what
//! these tests drive. `matrix_permissions_match_policy_source` (in
//! `matrix_coverage`) closes the remaining gap by checking each row's slice
//! still matches the policy it came from.
//!
//! The four traits declared in `ferriskey-domain` are additionally called for
//! real, because they are the only ones reachable from this crate — the other
//! fourteen live in crates that depend on this one.

use std::collections::HashSet;
use std::sync::Arc;

use chrono::Utc;
use ferriskey_domain::auth::Identity;
use ferriskey_domain::authentication::entities::AuthProtocol;
use ferriskey_domain::client::entities::{Client, ClientConfig, ClientType};
use ferriskey_domain::client::ports::{ClientPolicy, MockClientRepository};
use ferriskey_domain::common::policies::Policy;
use ferriskey_domain::realm::ports::RealmPolicy;
use ferriskey_domain::realm::scope::Unscoped;
use ferriskey_domain::realm::{Realm, RealmId};
use ferriskey_domain::role::entities::Role;
use ferriskey_domain::role::permission::Permissions;
use ferriskey_domain::role::ports::RolePolicy;
use ferriskey_domain::user::entities::User;
use ferriskey_domain::user::ports::{MockUserRepository, MockUserRoleRepository, UserPolicy};
use uuid::Uuid;

use ferriskey_authz::FerriskeyPolicy;
use ferriskey_authz::matrix::{AUTHZ_MATRIX, AuthzRow};

type Engine = FerriskeyPolicy<MockUserRepository, MockClientRepository, MockUserRoleRepository>;

const MASTER: &str = "master";
const TARGET: &str = "acme";
const FOREIGN: &str = "globex";

/// Permissions no matrix row accepts, used to build a subject that must be
/// denied everywhere. Asserted against each row rather than trusted.
const NEVER_GRANTED: [Permissions; 3] = [
    Permissions::QueryGroups,
    Permissions::ManageRoles,
    Permissions::CreateClient,
];

fn realm(name: &str) -> Realm {
    Realm {
        id: RealmId::new(Uuid::new_v4()),
        name: name.to_string(),
        display_name: None,
        settings: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn user_in(realm: &Realm) -> User {
    User {
        id: Uuid::new_v4(),
        realm_id: realm.id,
        client_id: None,
        username: "subject".to_string(),
        firstname: None,
        lastname: None,
        email: None,
        email_verified: true,
        enabled: true,
        roles: None,
        realm: Some(realm.clone()),
        required_actions: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        failed_login_attempts: 0,
        locked_until: None,
        locale: None,
    }
}

/// A role carrying `permissions`, attached to `client_id` when client-scoped.
fn role_with(realm_id: RealmId, client_id: Option<Uuid>, permissions: &[Permissions]) -> Role {
    Role {
        id: Uuid::new_v4(),
        name: "granting".to_string(),
        description: None,
        permissions: permissions.iter().map(|p| p.name()).collect(),
        realm_id,
        client_id,
        client: None,
        require_mfa: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// The `<target>-realm` client a master administrator's cross-realm roles hang
/// off, mirroring the convention `get_permission_for_target_realm` relies on.
fn realm_admin_client(realm_id: RealmId, target_realm_name: &str) -> Client {
    Client::new(ClientConfig {
        realm_id,
        name: format!("{target_realm_name}-realm"),
        client_id: format!("{target_realm_name}-realm"),
        secret: None,
        enabled: true,
        protocol: AuthProtocol::OpenIdConnect,
        public_client: false,
        service_account_enabled: false,
        client_type: ClientType::Confidential,
        direct_access_grants_enabled: None,
        oauth_device_code_grant_enabled: None,
        access_token_lifetime: None,
        refresh_token_lifetime: None,
        id_token_lifetime: None,
        temporary_token_lifetime: None,
    })
}

/// An engine whose subject holds `roles`, and which resolves the
/// `<target>-realm` client to `client` when asked.
fn engine(roles: Vec<Role>, client: Option<Client>) -> Engine {
    let mut user_role_repo = MockUserRoleRepository::new();
    user_role_repo.expect_get_user_roles().returning(move |_| {
        let roles = roles.clone();
        Box::pin(async move { Ok(roles) })
    });

    let mut client_repo = MockClientRepository::new();
    match client {
        Some(c) => {
            client_repo
                .expect_get_by_client_id()
                .returning(move |_, _| {
                    let c = c.clone();
                    Box::pin(async move { Ok(Unscoped::new(c)) })
                });
        }
        None => {
            client_repo.expect_get_by_client_id().returning(|_, _| {
                Box::pin(async move {
                    Err(ferriskey_domain::common::app_errors::CoreError::InvalidClient)
                })
            });
        }
    }

    FerriskeyPolicy::new(
        Arc::new(MockUserRepository::new()),
        Arc::new(client_repo),
        Arc::new(user_role_repo),
    )
}

/// The decision every `can_*` body computes: does the subject hold any of the
/// permissions the row accepts, for the target realm?
async fn decide(engine: &Engine, user: &User, target: &Realm, row: &AuthzRow) -> bool {
    let held = engine
        .get_permission_for_target_realm(user, target)
        .await
        .expect("resolving permissions must not fail on a well-formed fixture");
    Permissions::has_one_of_permissions(&held, row.permissions)
}

/// Runs `case` over every row and reports all failures at once, so one run
/// surfaces the full damage rather than the first row that breaks.
async fn for_every_row<F, Fut>(what: &str, case: F)
where
    F: Fn(&'static AuthzRow) -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    let mut failures = Vec::new();
    for row in AUTHZ_MATRIX {
        if let Err(e) = case(row).await {
            failures.push(format!(
                "{}:{} {} — {e}",
                row.file, row.line, row.service_fn
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failed on {}/{} rows:\n{}",
        what,
        failures.len(),
        AUTHZ_MATRIX.len(),
        failures.join("\n"),
    );
}

#[tokio::test]
async fn grants_when_the_subject_holds_any_listed_permission() {
    for_every_row("granting", |row| async move {
        for permission in row.permissions {
            let target = realm(TARGET);
            let user = user_in(&target);
            let e = engine(vec![role_with(target.id, None, &[*permission])], None);

            if !decide(&e, &user, &target, row).await {
                return Err(format!("holding {permission:?} did not grant"));
            }
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
async fn denies_when_the_subject_holds_no_listed_permission() {
    for_every_row("denying", |row| async move {
        let listed: HashSet<_> = row.permissions.iter().collect();
        let unlisted: Vec<Permissions> = NEVER_GRANTED
            .iter()
            .filter(|p| !listed.contains(p))
            .copied()
            .collect();
        if unlisted.is_empty() {
            return Err("no permission left outside the row to test denial with".into());
        }

        let target = realm(TARGET);
        let user = user_in(&target);
        let e = engine(vec![role_with(target.id, None, &unlisted)], None);

        if decide(&e, &user, &target, row).await {
            return Err(format!("holding only {unlisted:?} granted anyway"));
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
async fn denies_when_the_subject_has_no_role() {
    for_every_row("denying roleless", |row| async move {
        let target = realm(TARGET);
        let user = user_in(&target);
        let e = engine(vec![], None);

        if decide(&e, &user, &target, row).await {
            return Err("a subject with no role was granted".into());
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
async fn denies_a_subject_from_an_unrelated_realm() {
    for_every_row("realm isolation", |row| async move {
        let target = realm(TARGET);
        let foreign = realm(FOREIGN);
        let user = user_in(&foreign);
        // Every permission the row accepts, but held in the wrong realm.
        let e = engine(vec![role_with(foreign.id, None, row.permissions)], None);

        // A realm the subject cannot see is answered as absent, not as a
        // bare denial, so its existence does not leak.
        match e.get_permission_for_target_realm(&user, &target).await {
            Err(ferriskey_domain::common::app_errors::CoreError::NotFound) => Ok(()),
            Err(other) => Err(format!(
                "a subject of realm {FOREIGN} on realm {TARGET}: expected NotFound, got {other:?}"
            )),
            Ok(_) => Err(format!(
                "a subject of realm {FOREIGN} resolved permissions on realm {TARGET}"
            )),
        }
    })
    .await;
}

#[tokio::test]
async fn master_realm_level_roles_do_not_reach_another_realm() {
    // The cross-realm rule: a master administrator's realm-level roles grant
    // nothing on another realm. Only a role attached to the `<target>-realm`
    // client does. Getting this backwards would hand every master user full
    // authority over every tenant.
    for_every_row("cross-realm isolation", |row| async move {
        let master = realm(MASTER);
        let target = realm(TARGET);
        let user = user_in(&master);
        let client = realm_admin_client(master.id, TARGET);

        // Realm-level role (client_id: None) — not scoped to the target realm.
        let e = engine(
            vec![role_with(master.id, None, row.permissions)],
            Some(client),
        );

        if decide(&e, &user, &target, row).await {
            return Err("a master realm-level role granted on another realm".into());
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
async fn master_grants_through_the_target_realm_client_role() {
    for_every_row("cross-realm grant", |row| async move {
        let master = realm(MASTER);
        let target = realm(TARGET);
        let user = user_in(&master);
        let client = realm_admin_client(master.id, TARGET);

        let e = engine(
            vec![role_with(master.id, Some(client.id), row.permissions)],
            Some(client),
        );

        if !decide(&e, &user, &target, row).await {
            return Err("a role on the <target>-realm client did not grant".into());
        }
        Ok(())
    })
    .await;
}

// --- The four traits reachable from this crate, called for real -------------
//
// These prove the `can_*` wrappers actually test the slice the matrix records,
// rather than merely that the slice evaluates correctly in isolation.

fn row_for(service_fn: &str, policy_fn: &str) -> &'static AuthzRow {
    AUTHZ_MATRIX
        .iter()
        .find(|r| r.service_fn == service_fn && r.policy_fn == policy_fn)
        .unwrap_or_else(|| panic!("no matrix row for {service_fn} -> {policy_fn}"))
}

#[tokio::test]
async fn user_policy_agrees_with_its_matrix_row() {
    let row = row_for("create_user", "can_create_user");
    let target = realm(TARGET);
    let user = user_in(&target);

    for permission in row.permissions {
        let e = engine(vec![role_with(target.id, None, &[*permission])], None);
        assert!(
            e.can_create_user(&Identity::User(user.clone()), &target)
                .await
                .expect("policy must not error"),
            "can_create_user denied a subject holding {permission:?}",
        );
    }

    let e = engine(vec![role_with(target.id, None, &NEVER_GRANTED)], None);
    assert!(
        !e.can_create_user(&Identity::User(user), &target)
            .await
            .expect("policy must not error"),
        "can_create_user granted a subject holding none of its permissions",
    );
}

#[tokio::test]
async fn realm_role_and_client_policies_agree_with_their_matrix_rows() {
    let target = realm(TARGET);
    let user = user_in(&target);

    let cases: Vec<(&AuthzRow, &str)> = vec![
        (
            row_for("get_realm_by_name", "can_view_realm"),
            "can_view_realm",
        ),
        (row_for("create_role", "can_create_role"), "can_create_role"),
        (
            row_for("create_client", "can_create_client"),
            "can_create_client",
        ),
    ];

    for (row, label) in cases {
        for permission in row.permissions {
            let e = engine(vec![role_with(target.id, None, &[*permission])], None);
            let identity = Identity::User(user.clone());
            let granted = match label {
                "can_view_realm" => e.can_view_realm(&identity, &target).await,
                "can_create_role" => e.can_create_role(&identity, &target).await,
                _ => e.can_create_client(&identity, &target).await,
            }
            .expect("policy must not error");

            assert!(granted, "{label} denied a subject holding {permission:?}");
        }
    }
}

#[tokio::test]
async fn viewing_your_own_permissions_needs_no_permission() {
    // The single self-bypass in the codebase, recorded as `self_bypass` on the
    // matrix row. A subject reading its own permissions is allowed before any
    // permission is looked at — an engine reproducing only the permission slice
    // would deny this, silently removing an access that works today.
    let row = row_for("get_user_permissions", "can_view_user_permissions");
    assert!(row.self_bypass, "the matrix should record this bypass");

    let target = realm(TARGET);
    let user = user_in(&target);
    let e = engine(vec![], None); // no role at all

    assert!(
        e.can_view_user_permissions(&Identity::User(user.clone()), &target, user.id)
            .await
            .expect("policy must not error"),
        "a subject could not read its own permissions",
    );

    assert!(
        !e.can_view_user_permissions(&Identity::User(user), &target, Uuid::new_v4())
            .await
            .expect("policy must not error"),
        "the bypass leaked to another subject's permissions",
    );
}

#[tokio::test]
async fn only_one_row_carries_a_self_bypass() {
    // If a second bypass appears, it must be a deliberate decision reviewed
    // here — not something that slipped in with a policy edit.
    let bypasses: Vec<_> = AUTHZ_MATRIX
        .iter()
        .filter(|r| r.self_bypass)
        .map(|r| format!("{} -> {}", r.service_fn, r.policy_fn))
        .collect();

    assert_eq!(
        bypasses,
        vec!["get_user_permissions -> can_view_user_permissions"],
        "the set of self-bypassing policies changed",
    );
}

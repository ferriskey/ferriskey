//! Fixtures and mock builders shared by the aegis service unit tests.
//!
//! Both the aegis ports and the foreign domain ports are mocked with
//! `mockall::automock`, so nothing here duplicates a trait definition. The
//! builders only pre-program the handful of calls the authorization policy makes
//! on the way to the code under test.
//!
//! [`row_in_realm`] and [`row_in_scope`] are the load-bearing pieces: they
//! reproduce the `WHERE realm_id = $1` / `WHERE client_scope_id = $1` of the SQL
//! adapters, so a service that hands the repository the wrong realm — or reaches
//! a mapper through the wrong scope — fails the test instead of silently
//! returning another tenant's row.

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use ferriskey_domain::client::entities::{Client, ClientType, MaintenanceSessionStrategy};
use ferriskey_domain::client::ports::MockClientRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::policies::FerriskeyPolicy;
use ferriskey_domain::realm::ports::MockRealmRepository;
use ferriskey_domain::realm::{Realm, RealmId};
use ferriskey_domain::role::entities::Role;
use ferriskey_domain::user::entities::User;
use ferriskey_domain::user::ports::{MockUserRepository, MockUserRoleRepository};

use crate::entities::{ClientScope, ProtocolMapper, ScopeType};

// ─── fixtures ────────────────────────────────────────────────────────────────

pub(crate) fn make_realm(name: &str) -> Realm {
    Realm {
        id: RealmId::new(Uuid::new_v4()),
        name: name.to_string(),
        display_name: None,
        settings: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// A user of `realm`, holding a role that grants `manage_client_scopes` — so the
/// authorization policy passes and the tests observe the realm binding itself.
pub(crate) fn make_user(realm: &Realm) -> User {
    User {
        id: Uuid::new_v4(),
        realm_id: realm.id,
        client_id: None,
        username: "admin".to_string(),
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
    }
}

pub(crate) fn make_admin_role(realm_id: RealmId) -> Role {
    Role {
        id: Uuid::new_v4(),
        name: "scope-admin".to_string(),
        description: None,
        permissions: vec!["manage_client_scopes".to_string()],
        realm_id,
        client_id: None,
        client: None,
        require_mfa: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub(crate) fn make_client(id: Uuid, realm_id: RealmId) -> Client {
    Client {
        id,
        enabled: true,
        client_id: "app".to_string(),
        secret: None,
        realm_id,
        protocol: "openid-connect".to_string(),
        public_client: false,
        service_account_enabled: false,
        direct_access_grants_enabled: false,
        oauth_device_code_grant_enabled: false,
        require_pkce: false,
        client_type: ClientType::Confidential,
        name: "app".to_string(),
        redirect_uris: None,
        access_token_lifetime: None,
        refresh_token_lifetime: None,
        id_token_lifetime: None,
        temporary_token_lifetime: None,
        maintenance_enabled: false,
        maintenance_reason: None,
        maintenance_session_strategy: MaintenanceSessionStrategy::Expire,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub(crate) fn make_scope(realm_id: RealmId) -> ClientScope {
    ClientScope {
        id: Uuid::new_v4(),
        realm_id,
        name: "profile".to_string(),
        description: None,
        protocol: "openid-connect".to_string(),
        default_scope_type: ScopeType::Default,
        attributes: None,
        protocol_mappers: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub(crate) fn make_mapper(client_scope_id: Uuid) -> ProtocolMapper {
    ProtocolMapper {
        id: Uuid::new_v4(),
        client_scope_id,
        name: "forged-roles".to_string(),
        mapper_type: "oidc-hardcoded-claim-mapper".to_string(),
        config: serde_json::json!({ "claim.name": "realm_access.roles" }),
        created_at: Utc::now(),
    }
}

/// Emulates the realm-bound `SELECT … WHERE realm_id = $1 AND id = $2` of the
/// SQL adapter: a scope of another realm matches no row.
pub(crate) fn row_in_realm(
    scope: &ClientScope,
    realm_id: RealmId,
    id: Uuid,
) -> Option<ClientScope> {
    (scope.realm_id == realm_id && scope.id == id).then(|| scope.clone())
}

/// Same thing one level down: a mapper is only reachable through its own scope.
pub(crate) fn row_in_scope(
    mapper: &ProtocolMapper,
    client_scope_id: Uuid,
    id: Uuid,
) -> Option<ProtocolMapper> {
    (mapper.client_scope_id == client_scope_id && mapper.id == id).then(|| mapper.clone())
}

// ─── mocks of the foreign domain ports ───────────────────────────────────────

/// Resolves realms by name, the way the HTTP path realm is resolved.
pub(crate) fn mock_realm_repository(realms: Vec<Realm>) -> MockRealmRepository {
    let mut repository = MockRealmRepository::new();
    repository.expect_get_by_name().returning(move |name| {
        let found = realms.iter().find(|realm| realm.name == name).cloned();
        Box::pin(async move { Ok(found) })
    });
    repository
}

/// Hands the policy the roles that grant the scope permissions.
pub(crate) fn mock_user_role_repository(roles: Vec<Role>) -> MockUserRoleRepository {
    let mut repository = MockUserRoleRepository::new();
    repository.expect_get_user_roles().returning(move |_| {
        let roles = roles.clone();
        Box::pin(async move { Ok(roles) })
    });
    repository
}

/// Emulates the realm-bound client lookup of `ClientRepository::get_by_id`:
/// an id that belongs to another realm matches no row, exactly like the
/// `WHERE realm_id = $1` in the SQL adapter.
pub(crate) fn mock_client_repository(clients: Vec<Client>) -> MockClientRepository {
    let mut repository = MockClientRepository::new();
    repository
        .expect_get_by_id()
        .returning(move |realm_id, id| {
            let found = clients
                .iter()
                .find(|client| client.id == id && client.realm_id == realm_id)
                .cloned();
            Box::pin(async move { found.ok_or(CoreError::NotFound) })
        });
    repository
}

pub(crate) type TestPolicy =
    FerriskeyPolicy<MockUserRepository, MockClientRepository, MockUserRoleRepository>;

/// The tests authenticate as `Identity::User`, which the policy resolves from
/// the identity itself — the user repository is never called, so it carries no
/// expectation and any call would fail the test.
pub(crate) fn mock_policy(
    roles: Vec<Role>,
    client_repository: Arc<MockClientRepository>,
) -> Arc<TestPolicy> {
    Arc::new(FerriskeyPolicy::new(
        Arc::new(MockUserRepository::new()),
        client_repository,
        Arc::new(mock_user_role_repository(roles)),
    ))
}

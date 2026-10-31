use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use ferriskey_domain::realm::scope::{RealmScope, Scoped, Unscoped};
use ferriskey_domain::realm::{Realm, RealmId};
use ferriskey_domain::role::entities::Role;
use ferriskey_domain::user::entities::User;
use uuid::Uuid;

use super::TokenState;

const TTL: Duration = Duration::from_secs(60);

pub(super) fn synced_state() -> Arc<TokenState> {
    let (state, _feed) = TokenState::new(TTL);
    state.set_synced(true);
    state
}

pub(super) fn unsynced_state() -> Arc<TokenState> {
    TokenState::new(TTL).0
}

pub(super) fn disabled_state() -> Arc<TokenState> {
    let (state, _feed) = TokenState::new(Duration::ZERO);
    state.set_synced(true);
    state
}

fn realm() -> Realm {
    Realm::new("subject".to_string())
}

pub(super) fn user() -> User {
    let now = Utc::now();
    User {
        id: Uuid::new_v4(),
        realm_id: realm().id,
        client_id: None,
        username: "subject".to_string(),
        firstname: None,
        lastname: None,
        email: None,
        email_verified: false,
        enabled: true,
        roles: None,
        realm: None,
        required_actions: Vec::new(),
        created_at: now,
        updated_at: now,
        failed_login_attempts: 0,
        locked_until: None,
        locale: None,
        federation_provider_id: None,
    }
}

pub(super) fn role() -> Role {
    let now = Utc::now();
    Role {
        id: Uuid::new_v4(),
        name: "operator".to_string(),
        description: None,
        permissions: Vec::new(),
        realm_id: RealmId::new(Uuid::new_v4()),
        client_id: None,
        client: None,
        require_mfa: false,
        created_at: now,
        updated_at: now,
    }
}

pub(super) fn scoped<T: ferriskey_domain::realm::scope::RealmOwned>(value: T) -> Scoped<T> {
    let mut realm = realm();
    realm.id = value.realm_id();
    Unscoped::new(value)
        .in_realm(&RealmScope::from_realm(realm))
        .expect("a value of its own realm")
}

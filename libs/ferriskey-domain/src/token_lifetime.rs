use crate::client::entities::Client;
use crate::realm::RealmSetting;

pub struct TokenLifetimes {
    pub access_token: i64,
    pub refresh_token: i64,
    pub id_token: i64,
    pub temporary_token: i64,
}

impl TokenLifetimes {
    /// Resolve lifetimes: client override > realm default.
    pub fn resolve(realm: &RealmSetting, client: &Client) -> Self {
        Self {
            access_token: client
                .access_token_lifetime
                .unwrap_or(realm.access_token_lifetime),
            refresh_token: client
                .refresh_token_lifetime
                .unwrap_or(realm.refresh_token_lifetime),
            id_token: client.id_token_lifetime.unwrap_or(realm.id_token_lifetime),
            temporary_token: client
                .temporary_token_lifetime
                .unwrap_or(realm.temporary_token_lifetime),
        }
    }

    /// Use realm defaults only (no client override).
    pub fn from_realm(realm: &RealmSetting) -> Self {
        Self {
            access_token: realm.access_token_lifetime,
            refresh_token: realm.refresh_token_lifetime,
            id_token: realm.id_token_lifetime,
            temporary_token: realm.temporary_token_lifetime,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::entities::{Client, ClientConfig, ClientType};
    use crate::realm::{RealmId, RealmSetting};

    fn test_realm_setting() -> RealmSetting {
        RealmSetting::new(RealmId::default(), Some("RS256".to_string()))
    }

    fn test_client_config(realm_id: RealmId) -> ClientConfig {
        ClientConfig {
            realm_id,
            name: "test".to_string(),
            client_id: "test-client".to_string(),
            secret: None,
            enabled: true,
            protocol: "openid-connect".to_string(),
            public_client: true,
            service_account_enabled: false,
            client_type: ClientType::Public,
            direct_access_grants_enabled: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
        }
    }

    #[test]
    fn from_realm_uses_realm_defaults() {
        let realm = test_realm_setting();
        let lifetimes = TokenLifetimes::from_realm(&realm);

        assert_eq!(lifetimes.access_token, 300);
        assert_eq!(lifetimes.refresh_token, 86400);
        assert_eq!(lifetimes.id_token, 300);
        assert_eq!(lifetimes.temporary_token, 300);
    }

    #[test]
    fn resolve_uses_realm_when_client_has_no_override() {
        let realm = test_realm_setting();
        let client = Client::new(test_client_config(realm.realm_id));
        let lifetimes = TokenLifetimes::resolve(&realm, &client);

        assert_eq!(lifetimes.access_token, 300);
        assert_eq!(lifetimes.refresh_token, 86400);
    }

    #[test]
    fn resolve_uses_client_override() {
        let realm = test_realm_setting();
        let mut client = Client::new(test_client_config(realm.realm_id));
        client.access_token_lifetime = Some(600);
        client.refresh_token_lifetime = Some(3600);

        let lifetimes = TokenLifetimes::resolve(&realm, &client);

        assert_eq!(lifetimes.access_token, 600);
        assert_eq!(lifetimes.refresh_token, 3600);
        assert_eq!(lifetimes.id_token, 300);
        assert_eq!(lifetimes.temporary_token, 300);
    }
}

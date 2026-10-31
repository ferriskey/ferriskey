use std::sync::Arc;

use chrono::Duration;
use ferriskey_security::jwt::ports::SigningKeyRepository;
use ferriskey_security::jwt::signing_keys::{SigningKey, retirement_grace};
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;

use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::entities::Client;
use crate::domain::client::ports::ClientRepository;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::policies::{FerriskeyPolicy, ensure_policy};
use crate::domain::jwt::entities::{
    DEFAULT_ACCESS_TOKEN_LIFETIME, DEFAULT_ID_TOKEN_LIFETIME, DEFAULT_REFRESH_TOKEN_LIFETIME,
    DEFAULT_TEMPORARY_TOKEN_LIFETIME,
};
use crate::domain::realm::entities::{RealmScope, RealmSetting};
use crate::domain::realm::ports::{RealmPolicy, RealmRepository, SigningKeyService};
use crate::domain::seawatch::{
    EventStatus, SecurityEvent, SecurityEventRepository, SecurityEventType,
};
use crate::domain::user::ports::{UserRepository, UserRoleRepository};

const SIGNING_KEY_TARGET: &str = "signing_key";

/// Every lifetime a token signed by the realm key can have: the realm defaults
/// and the overrides its clients set.
pub(crate) fn signed_token_lifetimes(
    settings: Option<&RealmSetting>,
    clients: &[Client],
) -> Vec<i64> {
    let realm = match settings {
        Some(settings) => [
            settings.access_token_lifetime,
            settings.refresh_token_lifetime,
            settings.id_token_lifetime,
            settings.temporary_token_lifetime,
        ],
        None => [
            DEFAULT_ACCESS_TOKEN_LIFETIME,
            DEFAULT_REFRESH_TOKEN_LIFETIME,
            DEFAULT_ID_TOKEN_LIFETIME,
            DEFAULT_TEMPORARY_TOKEN_LIFETIME,
        ],
    };

    let overrides = clients.iter().flat_map(|client| {
        [
            client.access_token_lifetime,
            client.refresh_token_lifetime,
            client.id_token_lifetime,
            client.temporary_token_lifetime,
        ]
        .into_iter()
        .flatten()
    });

    realm.into_iter().chain(overrides).collect()
}

#[derive(Clone, Debug)]
pub struct SigningKeyServiceImpl<R, C, K, SE, U, UR>
where
    R: RealmRepository,
    C: ClientRepository,
    K: SigningKeyRepository,
    SE: SecurityEventRepository,
    U: UserRepository,
    UR: UserRoleRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) signing_key_repository: Arc<K>,
    pub(crate) security_event_repository: Arc<SE>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, C, K, SE, U, UR> SigningKeyServiceImpl<R, C, K, SE, U, UR>
where
    R: RealmRepository,
    C: ClientRepository,
    K: SigningKeyRepository,
    SE: SecurityEventRepository,
    U: UserRepository,
    UR: UserRoleRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        client_repository: Arc<C>,
        signing_key_repository: Arc<K>,
        security_event_repository: Arc<SE>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            signing_key_repository,
            security_event_repository,
            policy,
        }
    }

    async fn retirement_grace(&self, scope: &RealmScope) -> Result<Duration, CoreError> {
        let settings = self.realm_repository.get_realm_settings(scope.id()).await?;
        let clients = self.client_repository.get_by_realm_id(scope.id()).await?;

        Ok(retirement_grace(signed_token_lifetimes(
            settings.as_ref(),
            &clients,
        )))
    }

    async fn record(
        &self,
        scope: &RealmScope,
        identity: &Identity,
        event_type: SecurityEventType,
        key: &SigningKey,
    ) -> Result<(), CoreError> {
        self.security_event_repository
            .store_event(
                SecurityEvent::new(scope.id(), event_type, EventStatus::Success, identity.id())
                    .with_target(SIGNING_KEY_TARGET.to_string(), key.kid, None)
                    .with_details(json!({
                        "kid": key.kid,
                        "status": key.status,
                        "retire_at": key.retire_at,
                    })),
            )
            .await
    }
}

impl<R, C, K, SE, U, UR> SigningKeyService for SigningKeyServiceImpl<R, C, K, SE, U, UR>
where
    R: RealmRepository,
    C: ClientRepository,
    K: SigningKeyRepository,
    SE: SecurityEventRepository,
    U: UserRepository,
    UR: UserRoleRepository,
{
    #[instrument(skip(self, identity))]
    async fn list_signing_keys(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<SigningKey>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy.can_view_realm(&identity, scope.realm()).await,
            "insufficient permissions",
        )?;

        self.signing_key_repository
            .list_keys(scope.id())
            .await
            .map_err(|_| CoreError::RealmKeyNotFound)
    }

    #[instrument(skip(self, identity))]
    async fn rotate_signing_key(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<SigningKey, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy.can_update_realm(&identity, scope.realm()).await,
            "insufficient permissions",
        )?;

        let grace = self.retirement_grace(&scope).await?;
        let key = self
            .signing_key_repository
            .rotate_key(scope.id(), grace)
            .await
            .map_err(|error| CoreError::InvalidKey(error.to_string()))?;

        self.record(
            &scope,
            &identity,
            SecurityEventType::SigningKeyRotated,
            &key,
        )
        .await?;

        Ok(key)
    }

    #[instrument(skip(self, identity))]
    async fn revoke_signing_key(
        &self,
        identity: Identity,
        realm_name: String,
        kid: Uuid,
    ) -> Result<SigningKey, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        ensure_policy(
            self.policy.can_update_realm(&identity, scope.realm()).await,
            "insufficient permissions",
        )?;

        let key = self
            .signing_key_repository
            .revoke_key(scope.id(), kid)
            .await
            .map_err(|error| CoreError::InvalidKey(error.to_string()))?
            .ok_or(CoreError::NotFound)?;

        self.record(
            &scope,
            &identity,
            SecurityEventType::SigningKeyRevoked,
            &key,
        )
        .await?;

        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::authentication::entities::AuthProtocol;
    use crate::domain::client::entities::{ClientConfig, ClientType};
    use crate::domain::realm::entities::RealmId;

    fn client(access_token_lifetime: Option<i64>, refresh_token_lifetime: Option<i64>) -> Client {
        Client::new(ClientConfig {
            realm_id: RealmId::default(),
            name: "app".to_string(),
            client_id: "app".to_string(),
            secret: None,
            enabled: true,
            protocol: AuthProtocol::OpenIdConnect,
            public_client: true,
            service_account_enabled: false,
            client_type: ClientType::Public,
            direct_access_grants_enabled: None,
            oauth_device_code_grant_enabled: None,
            token_exchange_enabled: None,
            access_token_lifetime,
            refresh_token_lifetime,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
        })
    }

    #[test]
    fn a_realm_without_settings_uses_the_default_lifetimes() {
        let lifetimes = signed_token_lifetimes(None, &[]);

        assert_eq!(
            lifetimes.into_iter().max(),
            Some(DEFAULT_REFRESH_TOKEN_LIFETIME)
        );
    }

    #[test]
    fn a_client_override_longer_than_the_realm_extends_the_grace() {
        let clients = [client(Some(600), None), client(None, Some(7 * 86_400))];

        let grace = retirement_grace(signed_token_lifetimes(None, &clients));

        assert_eq!(grace, Duration::seconds(7 * 86_400 + 60));
    }
}

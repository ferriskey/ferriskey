use std::sync::Arc;
use std::time::Duration;

use ferriskey_domain::client::entities::{
    Client, ClientFilter, ClientRegistrationSource, ClientSortField,
};
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::client::value_objects::{CreateClientRequest, UpdateClientRequest};
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::{RealmScope, Scoped, Unscoped};
use uuid::Uuid;

use crate::infrastructure::common::{Retention, TtlCache};

#[derive(Debug, Clone)]
pub struct CachedClientRepository<R> {
    inner: R,
    by_client_id: Arc<TtlCache<(Uuid, String), Client>>,
}

impl<R> CachedClientRepository<R> {
    pub fn new(inner: R, ttl: Duration) -> Self {
        Self {
            inner,
            by_client_id: Arc::new(TtlCache::new(ttl)),
        }
    }
}

impl<R: ClientRepository> ClientRepository for CachedClientRepository<R> {
    async fn create_client(&self, data: CreateClientRequest) -> Result<Client, CoreError> {
        let created = self.inner.create_client(data).await;
        self.by_client_id.clear();
        created
    }

    async fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: RealmId,
    ) -> Result<Unscoped<Client>, CoreError> {
        let key = (Uuid::from(realm_id), client_id);
        let client = self
            .by_client_id
            .load(&key, || async {
                let client = self
                    .inner
                    .get_by_client_id(key.1.clone(), realm_id)
                    .await?
                    .across_realms();

                Ok::<_, CoreError>((client, Retention::Keep))
            })
            .await?;

        Ok(Unscoped::new(client))
    }

    async fn get_by_id(&self, realm_id: RealmId, id: Uuid) -> Result<Unscoped<Client>, CoreError> {
        self.inner.get_by_id(realm_id, id).await
    }

    async fn get_by_realm_id(&self, realm_id: RealmId) -> Result<Vec<Client>, CoreError> {
        self.inner.get_by_realm_id(realm_id).await
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<ClientFilter, ClientSortField>,
    ) -> Result<Page<Client>, CoreError> {
        self.inner.list(scope, request).await
    }

    async fn update_client(
        &self,
        client: &Scoped<Client>,
        data: UpdateClientRequest,
    ) -> Result<Client, CoreError> {
        let updated = self.inner.update_client(client, data).await;
        self.by_client_id.clear();
        updated
    }

    async fn delete_by_id(&self, client: &Scoped<Client>) -> Result<(), CoreError> {
        let deleted = self.inner.delete_by_id(client).await;
        self.by_client_id.clear();
        deleted
    }

    async fn count_by_source(
        &self,
        realm_id: RealmId,
        source: ClientRegistrationSource,
    ) -> Result<u64, CoreError> {
        self.inner.count_by_source(realm_id, source).await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;
    use ferriskey_domain::authentication::entities::AuthProtocol;
    use ferriskey_domain::client::entities::{Client, ClientConfig, ClientType};
    use ferriskey_domain::client::ports::{ClientRepository, MockClientRepository};
    use ferriskey_domain::realm::scope::{RealmScope, Unscoped};
    use ferriskey_domain::realm::{Realm, RealmId};
    use uuid::Uuid;

    use super::CachedClientRepository;

    fn realm() -> Realm {
        Realm {
            id: RealmId::from(Uuid::new_v4()),
            name: "acme".to_string(),
            display_name: None,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn client(realm_id: RealmId) -> Client {
        Client::new(ClientConfig {
            realm_id,
            name: "console".to_string(),
            client_id: "console".to_string(),
            secret: None,
            enabled: true,
            protocol: AuthProtocol::OpenIdConnect,
            public_client: true,
            service_account_enabled: false,
            client_type: ClientType::Public,
            direct_access_grants_enabled: None,
            oauth_device_code_grant_enabled: None,
            token_exchange_enabled: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
        })
    }

    fn inner_returning(client: Client, calls: usize) -> MockClientRepository {
        let mut inner = MockClientRepository::new();
        inner
            .expect_get_by_client_id()
            .times(calls)
            .returning(move |_, _| {
                let client = client.clone();
                Box::pin(async move { Ok(Unscoped::new(client)) })
            });
        inner
    }

    #[tokio::test]
    async fn a_client_read_within_the_ttl_is_served_from_memory() {
        let realm = realm();
        let cached = CachedClientRepository::new(
            inner_returning(client(realm.id), 1),
            Duration::from_secs(5),
        );
        let scope = RealmScope::from_realm(realm.clone());

        let first = cached
            .get_by_client_id("console".to_string(), realm.id)
            .await
            .unwrap()
            .in_realm(&scope)
            .unwrap()
            .into_inner();
        let second = cached
            .get_by_client_id("console".to_string(), realm.id)
            .await
            .unwrap()
            .in_realm(&scope)
            .unwrap()
            .into_inner();

        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn concurrent_reads_of_an_uncached_client_hit_the_store_once() {
        let realm = realm();
        let stored = client(realm.id);
        let mut inner = MockClientRepository::new();
        inner
            .expect_get_by_client_id()
            .times(1)
            .returning(move |_, _| {
                let stored = stored.clone();
                Box::pin(async move {
                    for _ in 0..64 {
                        tokio::task::yield_now().await;
                    }
                    Ok(Unscoped::new(stored))
                })
            });
        let cached =
            std::sync::Arc::new(CachedClientRepository::new(inner, Duration::from_secs(5)));

        let reads: Vec<_> = (0..16)
            .map(|_| {
                let cached = std::sync::Arc::clone(&cached);
                let realm_id = realm.id;
                tokio::spawn(async move {
                    cached
                        .get_by_client_id("console".to_string(), realm_id)
                        .await
                })
            })
            .collect();

        for read in reads {
            assert!(read.await.unwrap().is_ok());
        }
    }

    #[tokio::test]
    async fn the_same_client_id_in_another_realm_is_a_different_entry() {
        let realm = realm();
        let other_realm = RealmId::from(Uuid::new_v4());
        let cached = CachedClientRepository::new(
            inner_returning(client(realm.id), 2),
            Duration::from_secs(5),
        );

        let _ = cached
            .get_by_client_id("console".to_string(), realm.id)
            .await
            .unwrap();
        let _ = cached
            .get_by_client_id("console".to_string(), other_realm)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn an_expired_entry_is_read_again() {
        let realm = realm();
        let cached =
            CachedClientRepository::new(inner_returning(client(realm.id), 2), Duration::ZERO);

        let _ = cached
            .get_by_client_id("console".to_string(), realm.id)
            .await
            .unwrap();
        let _ = cached
            .get_by_client_id("console".to_string(), realm.id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn a_write_invalidates_the_cached_client() {
        let realm = realm();
        let client = client(realm.id);
        let mut inner = inner_returning(client.clone(), 2);
        inner
            .expect_delete_by_id()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
        let cached = CachedClientRepository::new(inner, Duration::from_secs(5));
        let scoped = Unscoped::new(client)
            .in_realm(&RealmScope::from_realm(realm.clone()))
            .unwrap();

        let _ = cached
            .get_by_client_id("console".to_string(), realm.id)
            .await
            .unwrap();
        cached.delete_by_id(&scoped).await.unwrap();
        let _ = cached
            .get_by_client_id("console".to_string(), realm.id)
            .await
            .unwrap();
    }
}

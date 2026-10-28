use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use ferriskey_aegis::entities::{
    ClientScope, ClientScopeFilter, ClientScopeMapping, ClientScopeSet, ClientScopeSortField,
    ProtocolMapper,
};
use ferriskey_aegis::ports::{
    ClientScopeMappingRepository, ClientScopeRepository, ProtocolMapperRepository,
};
use ferriskey_aegis::value_objects::{
    CreateClientScopeRequest, CreateProtocolMapperRequest, UpdateClientScopeRequest,
    UpdateProtocolMapperRequest,
};
use ferriskey_domain::client::entities::Client;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::{RealmScope, Scoped, Unscoped};
use uuid::Uuid;

use crate::infrastructure::common::{Retention, TtlCache};

#[derive(Debug)]
pub struct AegisConfigCache {
    generation: AtomicU64,
    scope_sets: TtlCache<Uuid, ClientScopeSet>,
    mappers: TtlCache<Vec<Uuid>, HashMap<Uuid, Vec<ProtocolMapper>>>,
}

impl AegisConfigCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            generation: AtomicU64::new(0),
            scope_sets: TtlCache::new(ttl),
            mappers: TtlCache::new(ttl),
        }
    }

    pub fn clear(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.scope_sets.clear();
        self.mappers.clear();
    }

    fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    fn retention_since(&self, read_at: u64) -> Retention {
        if self.generation() == read_at {
            Retention::Keep
        } else {
            Retention::Own
        }
    }
}

fn mapper_key(scope_ids: &[Uuid]) -> Vec<Uuid> {
    let mut key = scope_ids.to_vec();
    key.sort_unstable();
    key.dedup();
    key
}

#[derive(Debug, Clone)]
pub struct CachedScopeMappingRepository<R> {
    inner: R,
    cache: Arc<AegisConfigCache>,
}

impl<R> CachedScopeMappingRepository<R> {
    pub fn new(inner: R, cache: Arc<AegisConfigCache>) -> Self {
        Self { inner, cache }
    }
}

impl<R: ClientScopeMappingRepository> ClientScopeMappingRepository
    for CachedScopeMappingRepository<R>
{
    async fn assign_scope_to_client(
        &self,
        client_id: Uuid,
        scope_id: Uuid,
        is_default: bool,
        is_optional: bool,
    ) -> Result<ClientScopeMapping, CoreError> {
        let assigned = self
            .inner
            .assign_scope_to_client(client_id, scope_id, is_default, is_optional)
            .await;
        self.cache.clear();
        assigned
    }

    async fn remove_scope_from_client(
        &self,
        client: &Scoped<Client>,
        client_scope: &Scoped<ClientScope>,
    ) -> Result<(), CoreError> {
        let removed = self
            .inner
            .remove_scope_from_client(client, client_scope)
            .await;
        self.cache.clear();
        removed
    }

    async fn get_client_scopes(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<ClientScopeMapping>, CoreError> {
        self.inner.get_client_scopes(client_id).await
    }

    async fn get_default_scopes(&self, client_id: Uuid) -> Result<Vec<ClientScope>, CoreError> {
        self.inner.get_default_scopes(client_id).await
    }

    async fn get_optional_scopes(&self, client_id: Uuid) -> Result<Vec<ClientScope>, CoreError> {
        self.inner.get_optional_scopes(client_id).await
    }

    async fn get_scope_set(&self, client_id: Uuid) -> Result<ClientScopeSet, CoreError> {
        let read_at = self.cache.generation();
        self.cache
            .scope_sets
            .load_while(
                &client_id,
                || self.cache.generation() == read_at,
                || async {
                    let set = self.inner.get_scope_set(client_id).await?;

                    Ok((set, self.cache.retention_since(read_at)))
                },
            )
            .await
    }
}

#[derive(Debug, Clone)]
pub struct CachedProtocolMapperRepository<R> {
    inner: R,
    cache: Arc<AegisConfigCache>,
}

impl<R> CachedProtocolMapperRepository<R> {
    pub fn new(inner: R, cache: Arc<AegisConfigCache>) -> Self {
        Self { inner, cache }
    }
}

impl<R: ProtocolMapperRepository> ProtocolMapperRepository for CachedProtocolMapperRepository<R> {
    async fn create(
        &self,
        payload: CreateProtocolMapperRequest,
    ) -> Result<ProtocolMapper, CoreError> {
        let created = self.inner.create(payload).await;
        self.cache.clear();
        created
    }

    async fn get_by_id(
        &self,
        client_scope: &Scoped<ClientScope>,
        id: Uuid,
    ) -> Result<Option<ProtocolMapper>, CoreError> {
        self.inner.get_by_id(client_scope, id).await
    }

    async fn get_by_scope_id(&self, scope_id: Uuid) -> Result<Vec<ProtocolMapper>, CoreError> {
        self.inner.get_by_scope_id(scope_id).await
    }

    async fn get_by_scope_ids(
        &self,
        scope_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<ProtocolMapper>>, CoreError> {
        let key = mapper_key(scope_ids);
        let read_at = self.cache.generation();
        self.cache
            .mappers
            .load_while(
                &key,
                || self.cache.generation() == read_at,
                || async {
                    let mappers = self.inner.get_by_scope_ids(scope_ids).await?;

                    Ok((mappers, self.cache.retention_since(read_at)))
                },
            )
            .await
    }

    async fn update_by_id(
        &self,
        client_scope: &Scoped<ClientScope>,
        id: Uuid,
        payload: UpdateProtocolMapperRequest,
    ) -> Result<ProtocolMapper, CoreError> {
        let updated = self.inner.update_by_id(client_scope, id, payload).await;
        self.cache.clear();
        updated
    }

    async fn delete_by_id(
        &self,
        client_scope: &Scoped<ClientScope>,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let deleted = self.inner.delete_by_id(client_scope, id).await;
        self.cache.clear();
        deleted
    }
}

#[derive(Debug, Clone)]
pub struct CachedClientScopeRepository<R> {
    inner: R,
    cache: Arc<AegisConfigCache>,
}

impl<R> CachedClientScopeRepository<R> {
    pub fn new(inner: R, cache: Arc<AegisConfigCache>) -> Self {
        Self { inner, cache }
    }
}

impl<R: ClientScopeRepository> ClientScopeRepository for CachedClientScopeRepository<R> {
    async fn create(&self, payload: CreateClientScopeRequest) -> Result<ClientScope, CoreError> {
        let created = self.inner.create(payload).await;
        self.cache.clear();
        created
    }

    async fn get_by_id(
        &self,
        realm_id: RealmId,
        id: Uuid,
    ) -> Result<Option<Unscoped<ClientScope>>, CoreError> {
        self.inner.get_by_id(realm_id, id).await
    }

    async fn find_by_realm_id(&self, realm_id: RealmId) -> Result<Vec<ClientScope>, CoreError> {
        self.inner.find_by_realm_id(realm_id).await
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<ClientScopeFilter, ClientScopeSortField>,
    ) -> Result<Page<ClientScope>, CoreError> {
        self.inner.list(scope, request).await
    }

    async fn find_by_name(
        &self,
        name: String,
        realm_id: RealmId,
    ) -> Result<Option<Unscoped<ClientScope>>, CoreError> {
        self.inner.find_by_name(name, realm_id).await
    }

    async fn protocol_mappers_of(
        &self,
        scope_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<ProtocolMapper>>, CoreError> {
        self.inner.protocol_mappers_of(scope_ids).await
    }

    async fn update_by_id(
        &self,
        client_scope: &Scoped<ClientScope>,
        payload: UpdateClientScopeRequest,
    ) -> Result<ClientScope, CoreError> {
        let updated = self.inner.update_by_id(client_scope, payload).await;
        self.cache.clear();
        updated
    }

    async fn delete_by_id(&self, client_scope: &Scoped<ClientScope>) -> Result<(), CoreError> {
        let deleted = self.inner.delete_by_id(client_scope).await;
        self.cache.clear();
        deleted
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use ferriskey_aegis::entities::{
        ClientScopeMapping, ClientScopeSet, ProtocolMapper, ScopeType,
    };
    use ferriskey_aegis::ports::{
        ClientScopeMappingRepository, MockClientScopeMappingRepository,
        MockProtocolMapperRepository, ProtocolMapperRepository,
    };
    use ferriskey_aegis::value_objects::CreateProtocolMapperRequest;
    use uuid::Uuid;

    use super::{AegisConfigCache, CachedProtocolMapperRepository, CachedScopeMappingRepository};

    fn mappings_returning_sets(calls: usize) -> MockClientScopeMappingRepository {
        let mut inner = MockClientScopeMappingRepository::new();
        inner
            .expect_get_scope_set()
            .times(calls)
            .returning(|_| Box::pin(async { Ok(ClientScopeSet::default()) }));
        inner
    }

    fn mappers_returning(calls: usize) -> MockProtocolMapperRepository {
        let mut inner = MockProtocolMapperRepository::new();
        inner
            .expect_get_by_scope_ids()
            .times(calls)
            .returning(|_| Box::pin(async { Ok(HashMap::new()) }));
        inner
    }

    fn mapper(scope_id: Uuid) -> ProtocolMapper {
        ProtocolMapper {
            id: Uuid::new_v4(),
            client_scope_id: scope_id,
            name: "email".to_string(),
            mapper_type: "oidc-usermodel-property-mapper".to_string(),
            config: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn a_scope_set_is_served_from_memory_within_the_ttl() {
        let cache = Arc::new(AegisConfigCache::new(Duration::from_secs(5)));
        let mappings = CachedScopeMappingRepository::new(mappings_returning_sets(1), cache);
        let client_id = Uuid::new_v4();

        let _ = mappings.get_scope_set(client_id).await.unwrap();
        let _ = mappings.get_scope_set(client_id).await.unwrap();
    }

    #[tokio::test]
    async fn a_zero_ttl_reads_the_scope_set_every_time() {
        let cache = Arc::new(AegisConfigCache::new(Duration::ZERO));
        let mappings = CachedScopeMappingRepository::new(mappings_returning_sets(2), cache);
        let client_id = Uuid::new_v4();

        let _ = mappings.get_scope_set(client_id).await.unwrap();
        let _ = mappings.get_scope_set(client_id).await.unwrap();
    }

    #[tokio::test]
    async fn assigning_a_scope_clears_the_cached_scope_sets() {
        let cache = Arc::new(AegisConfigCache::new(Duration::from_secs(5)));
        let mut inner = mappings_returning_sets(2);
        inner
            .expect_assign_scope_to_client()
            .times(1)
            .returning(|client_id, scope_id, _, _| {
                Box::pin(async move {
                    Ok(ClientScopeMapping {
                        client_id,
                        scope_id,
                        default_scope_type: ScopeType::Default,
                    })
                })
            });
        let mappings = CachedScopeMappingRepository::new(inner, cache);
        let client_id = Uuid::new_v4();

        let _ = mappings.get_scope_set(client_id).await.unwrap();
        mappings
            .assign_scope_to_client(client_id, Uuid::new_v4(), true, false)
            .await
            .unwrap();
        let _ = mappings.get_scope_set(client_id).await.unwrap();
    }

    #[tokio::test]
    async fn a_mapper_write_clears_the_cached_scope_sets_too() {
        let cache = Arc::new(AegisConfigCache::new(Duration::from_secs(5)));
        let mappings =
            CachedScopeMappingRepository::new(mappings_returning_sets(2), Arc::clone(&cache));
        let mut inner = MockProtocolMapperRepository::new();
        inner
            .expect_create()
            .times(1)
            .returning(|payload| Box::pin(async move { Ok(mapper(payload.client_scope_id)) }));
        let mappers = CachedProtocolMapperRepository::new(inner, cache);
        let client_id = Uuid::new_v4();

        let _ = mappings.get_scope_set(client_id).await.unwrap();
        mappers
            .create(CreateProtocolMapperRequest {
                client_scope_id: Uuid::new_v4(),
                name: "email".to_string(),
                mapper_type: "oidc-usermodel-property-mapper".to_string(),
                config: serde_json::json!({}),
            })
            .await
            .unwrap();
        let _ = mappings.get_scope_set(client_id).await.unwrap();
    }

    #[tokio::test]
    async fn a_read_overtaken_by_a_write_is_not_kept() {
        let cache = Arc::new(AegisConfigCache::new(Duration::from_secs(5)));
        let during_read = Arc::clone(&cache);
        let mut inner = MockClientScopeMappingRepository::new();
        inner.expect_get_scope_set().times(2).returning(move |_| {
            during_read.clear();
            Box::pin(async { Ok(ClientScopeSet::default()) })
        });
        let mappings = CachedScopeMappingRepository::new(inner, cache);
        let client_id = Uuid::new_v4();

        let _ = mappings.get_scope_set(client_id).await.unwrap();
        let _ = mappings.get_scope_set(client_id).await.unwrap();
    }

    #[tokio::test]
    async fn a_mapper_read_overtaken_by_a_write_is_not_kept() {
        let cache = Arc::new(AegisConfigCache::new(Duration::from_secs(5)));
        let during_read = Arc::clone(&cache);
        let mut inner = MockProtocolMapperRepository::new();
        inner
            .expect_get_by_scope_ids()
            .times(2)
            .returning(move |_| {
                during_read.clear();
                Box::pin(async { Ok(HashMap::new()) })
            });
        let mappers = CachedProtocolMapperRepository::new(inner, cache);
        let scope_id = Uuid::new_v4();

        let _ = mappers.get_by_scope_ids(&[scope_id]).await.unwrap();
        let _ = mappers.get_by_scope_ids(&[scope_id]).await.unwrap();
    }

    #[tokio::test]
    async fn mappers_are_cached_by_the_set_of_scope_ids_whatever_their_order() {
        let cache = Arc::new(AegisConfigCache::new(Duration::from_secs(5)));
        let mappers = CachedProtocolMapperRepository::new(mappers_returning(1), cache);
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());

        let _ = mappers.get_by_scope_ids(&[a, b]).await.unwrap();
        let _ = mappers.get_by_scope_ids(&[b, a, b]).await.unwrap();
    }

    #[tokio::test]
    async fn a_different_set_of_scopes_is_read_again() {
        let cache = Arc::new(AegisConfigCache::new(Duration::from_secs(5)));
        let mappers = CachedProtocolMapperRepository::new(mappers_returning(2), cache);
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());

        let _ = mappers.get_by_scope_ids(&[a]).await.unwrap();
        let _ = mappers.get_by_scope_ids(&[a, b]).await.unwrap();
    }
}

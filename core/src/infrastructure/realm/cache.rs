use std::sync::Arc;
use std::time::Duration;

use ferriskey_domain::realm::LoginAliases;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::locale::SupportedLocales;
use crate::domain::common::pagination::{Page, PageRequest};
use crate::domain::realm::entities::{Realm, RealmFilter, RealmId, RealmSetting, RealmSortField};
use crate::domain::realm::ports::RealmRepository;
use crate::infrastructure::common::{Retention, TtlCache};

#[derive(Debug, Clone)]
pub struct CachedRealmRepository<R> {
    inner: R,
    by_name: Arc<TtlCache<String, Option<Realm>>>,
    by_id: Arc<TtlCache<Uuid, Option<Realm>>>,
    settings: Arc<TtlCache<Uuid, Option<RealmSetting>>>,
}

fn retention_of<T>(found: &Option<T>) -> Retention {
    match found {
        Some(_) => Retention::Keep,
        None => Retention::Share,
    }
}

impl<R> CachedRealmRepository<R> {
    pub fn new(inner: R, ttl: Duration) -> Self {
        Self {
            inner,
            by_name: Arc::new(TtlCache::new(ttl)),
            by_id: Arc::new(TtlCache::new(ttl)),
            settings: Arc::new(TtlCache::new(ttl)),
        }
    }

    fn invalidate(&self) {
        self.by_name.clear();
        self.by_id.clear();
        self.settings.clear();
    }
}

impl<R: RealmRepository> RealmRepository for CachedRealmRepository<R> {
    async fn fetch_realm(&self) -> Result<Vec<Realm>, CoreError> {
        self.inner.fetch_realm().await
    }

    async fn list(
        &self,
        accessible: &[RealmId],
        request: &PageRequest<RealmFilter, RealmSortField>,
    ) -> Result<Page<Realm>, CoreError> {
        self.inner.list(accessible, request).await
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Realm>, CoreError> {
        self.by_name
            .load(name, || async {
                let realm = self.inner.get_by_name(name).await?;
                let retention = retention_of(&realm);

                Ok((realm, retention))
            })
            .await
    }

    async fn get_by_id(&self, realm_id: RealmId) -> Result<Option<Realm>, CoreError> {
        let key = Uuid::from(realm_id);
        self.by_id
            .load(&key, || async {
                let realm = self.inner.get_by_id(realm_id).await?;
                let retention = retention_of(&realm);

                Ok((realm, retention))
            })
            .await
    }

    async fn create_realm(
        &self,
        name: String,
        display_name: Option<String>,
    ) -> Result<Realm, CoreError> {
        let created = self.inner.create_realm(name, display_name).await;
        self.invalidate();
        created
    }

    async fn update_realm(
        &self,
        realm_name: String,
        name: String,
        display_name: Option<String>,
    ) -> Result<Realm, CoreError> {
        let updated = self
            .inner
            .update_realm(realm_name, name, display_name)
            .await;
        self.invalidate();
        updated
    }

    async fn delete_by_name(&self, name: &str) -> Result<(), CoreError> {
        let deleted = self.inner.delete_by_name(name).await;
        self.invalidate();
        deleted
    }

    async fn create_realm_settings(
        &self,
        realm_id: RealmId,
        algorithm: String,
    ) -> Result<RealmSetting, CoreError> {
        let created = self.inner.create_realm_settings(realm_id, algorithm).await;
        self.invalidate();
        created
    }

    #[allow(clippy::too_many_arguments)]
    async fn update_realm_setting(
        &self,
        realm_id: RealmId,
        algorithm: Option<String>,
        user_registration_enabled: Option<bool>,
        forgot_password_enabled: Option<bool>,
        remember_me_enabled: Option<bool>,
        magic_link_enabled: Option<bool>,
        magic_link_ttl: Option<u32>,
        passkey_enabled: Option<bool>,
        compass_enabled: Option<bool>,
        access_token_lifetime: Option<i64>,
        refresh_token_lifetime: Option<i64>,
        id_token_lifetime: Option<i64>,
        temporary_token_lifetime: Option<i64>,
        reset_password_template_id: Option<Option<Uuid>>,
        magic_link_template_id: Option<Option<Uuid>>,
        email_verification_template_id: Option<Option<Uuid>>,
        email_verification_enabled: Option<bool>,
        email_verification_ttl_hours: Option<i64>,
        lockout_threshold: Option<i32>,
        lockout_duration_seconds: Option<i32>,
        login_aliases: Option<LoginAliases>,
        locales: Option<SupportedLocales>,
        seawatch_pii_mode: Option<String>,
        seawatch_pseudo_key: Option<Option<String>>,
        require_mfa: Option<bool>,
        edit_username_enabled: Option<bool>,
        webhook_retry_max_attempts: Option<Option<i32>>,
        webhook_retry_base_delay_ms: Option<Option<i32>>,
        webhook_retry_max_delay_ms: Option<Option<i32>>,
        webhook_retry_max_total_delay_ms: Option<Option<i32>>,
        cimd_enabled: Option<bool>,
        dcr_enabled: Option<bool>,
        cimd_allowed_hosts: Option<Vec<String>>,
        allowed_resources: Option<Vec<String>>,
    ) -> Result<RealmSetting, CoreError> {
        let updated = self
            .inner
            .update_realm_setting(
                realm_id,
                algorithm,
                user_registration_enabled,
                forgot_password_enabled,
                remember_me_enabled,
                magic_link_enabled,
                magic_link_ttl,
                passkey_enabled,
                compass_enabled,
                access_token_lifetime,
                refresh_token_lifetime,
                id_token_lifetime,
                temporary_token_lifetime,
                reset_password_template_id,
                magic_link_template_id,
                email_verification_template_id,
                email_verification_enabled,
                email_verification_ttl_hours,
                lockout_threshold,
                lockout_duration_seconds,
                login_aliases,
                locales,
                seawatch_pii_mode,
                seawatch_pseudo_key,
                require_mfa,
                edit_username_enabled,
                webhook_retry_max_attempts,
                webhook_retry_base_delay_ms,
                webhook_retry_max_delay_ms,
                webhook_retry_max_total_delay_ms,
                cimd_enabled,
                dcr_enabled,
                cimd_allowed_hosts,
                allowed_resources,
            )
            .await;
        self.invalidate();
        updated
    }

    async fn get_realm_settings(
        &self,
        realm_id: RealmId,
    ) -> Result<Option<RealmSetting>, CoreError> {
        let key = Uuid::from(realm_id);
        self.settings
            .load(&key, || async {
                let settings = self.inner.get_realm_settings(realm_id).await?;
                let retention = retention_of(&settings);

                Ok((settings, retention))
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;
    use uuid::Uuid;

    use super::CachedRealmRepository;
    use crate::domain::realm::entities::{Realm, RealmId, RealmSetting};
    use crate::domain::realm::ports::{MockRealmRepository, RealmRepository};

    fn realm(name: &str) -> Realm {
        Realm {
            id: RealmId::from(Uuid::new_v4()),
            name: name.to_string(),
            display_name: None,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn inner_returning(realm: Option<Realm>, calls: usize) -> MockRealmRepository {
        let mut inner = MockRealmRepository::new();
        inner.expect_get_by_name().times(calls).returning(move |_| {
            let realm = realm.clone();
            Box::pin(async move { Ok(realm) })
        });
        inner
    }

    #[tokio::test]
    async fn a_realm_read_within_the_ttl_is_served_from_memory() {
        let cached = CachedRealmRepository::new(
            inner_returning(Some(realm("acme")), 1),
            Duration::from_secs(5),
        );

        let first = cached.get_by_name("acme").await.unwrap().unwrap();
        let second = cached.get_by_name("acme").await.unwrap().unwrap();

        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn a_missing_realm_is_not_cached() {
        let cached = CachedRealmRepository::new(inner_returning(None, 2), Duration::from_secs(5));

        assert!(cached.get_by_name("ghost").await.unwrap().is_none());
        assert!(cached.get_by_name("ghost").await.unwrap().is_none());
    }

    fn slow_inner_returning(realm: Option<Realm>) -> MockRealmRepository {
        let mut inner = MockRealmRepository::new();
        inner.expect_get_by_name().times(1).returning(move |_| {
            let realm = realm.clone();
            Box::pin(async move {
                for _ in 0..64 {
                    tokio::task::yield_now().await;
                }
                Ok(realm)
            })
        });
        inner
    }

    async fn read_concurrently(
        cached: CachedRealmRepository<MockRealmRepository>,
        name: &'static str,
    ) -> Vec<Option<Realm>> {
        let cached = std::sync::Arc::new(cached);
        let reads: Vec<_> = (0..16)
            .map(|_| {
                let cached = std::sync::Arc::clone(&cached);
                tokio::spawn(async move { cached.get_by_name(name).await })
            })
            .collect();

        let mut realms = Vec::new();
        for read in reads {
            realms.push(read.await.unwrap().unwrap());
        }
        realms
    }

    #[tokio::test]
    async fn concurrent_reads_of_an_uncached_realm_hit_the_store_once() {
        let cached = CachedRealmRepository::new(
            slow_inner_returning(Some(realm("acme"))),
            Duration::from_secs(5),
        );

        let realms = read_concurrently(cached, "acme").await;

        assert!(realms.iter().all(|realm| realm.is_some()));
    }

    #[tokio::test]
    async fn concurrent_reads_of_a_missing_realm_share_one_lookup() {
        let cached = CachedRealmRepository::new(slow_inner_returning(None), Duration::from_secs(5));

        let realms = read_concurrently(cached, "ghost").await;

        assert!(realms.iter().all(|realm| realm.is_none()));
    }

    #[tokio::test]
    async fn an_expired_entry_is_read_again() {
        let cached =
            CachedRealmRepository::new(inner_returning(Some(realm("acme")), 2), Duration::ZERO);

        cached.get_by_name("acme").await.unwrap();
        cached.get_by_name("acme").await.unwrap();
    }

    #[tokio::test]
    async fn a_write_invalidates_the_cached_realm() {
        let mut inner = inner_returning(Some(realm("acme")), 2);
        inner
            .expect_delete_by_name()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
        let cached = CachedRealmRepository::new(inner, Duration::from_secs(5));

        cached.get_by_name("acme").await.unwrap();
        cached.delete_by_name("acme").await.unwrap();
        cached.get_by_name("acme").await.unwrap();
    }

    #[tokio::test]
    async fn realm_settings_are_cached_by_realm() {
        let realm_id = RealmId::from(Uuid::new_v4());
        let mut inner = MockRealmRepository::new();
        inner
            .expect_get_realm_settings()
            .times(1)
            .returning(move |id| Box::pin(async move { Ok(Some(RealmSetting::new(id, None))) }));
        let cached = CachedRealmRepository::new(inner, Duration::from_secs(5));

        let first = cached.get_realm_settings(realm_id).await.unwrap().unwrap();
        let second = cached.get_realm_settings(realm_id).await.unwrap().unwrap();

        assert_eq!(first.realm_id, second.realm_id);
    }
}

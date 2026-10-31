use std::sync::Arc;

use ferriskey_domain::realm::LoginAliases;
use uuid::Uuid;

use super::state::TokenState;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::locale::SupportedLocales;
use crate::domain::common::pagination::{Page, PageRequest};
use crate::domain::realm::entities::{Realm, RealmFilter, RealmId, RealmSetting, RealmSortField};
use crate::domain::realm::ports::RealmRepository;

#[derive(Debug, Clone)]
pub struct InvalidatingRealmRepository<R> {
    inner: R,
    state: Arc<TokenState>,
}

impl<R> InvalidatingRealmRepository<R> {
    pub fn new(inner: R, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: RealmRepository> RealmRepository for InvalidatingRealmRepository<R> {
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
        self.inner.get_by_name(name).await
    }

    async fn get_by_id(&self, realm_id: RealmId) -> Result<Option<Realm>, CoreError> {
        self.inner.get_by_id(realm_id).await
    }

    async fn create_realm(
        &self,
        name: String,
        display_name: Option<String>,
    ) -> Result<Realm, CoreError> {
        self.inner.create_realm(name, display_name).await
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
        self.state.changed();
        updated
    }

    async fn delete_by_name(&self, name: &str) -> Result<(), CoreError> {
        let deleted = self.inner.delete_by_name(name).await;
        self.state.changed();
        deleted
    }

    async fn create_realm_settings(
        &self,
        realm_id: RealmId,
        algorithm: String,
    ) -> Result<RealmSetting, CoreError> {
        self.inner.create_realm_settings(realm_id, algorithm).await
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
        self.state.changed();
        updated
    }

    async fn get_realm_settings(
        &self,
        realm_id: RealmId,
    ) -> Result<Option<RealmSetting>, CoreError> {
        self.inner.get_realm_settings(realm_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::realm::ports::MockRealmRepository;
    use crate::infrastructure::token_state::test_support::synced_state;

    #[tokio::test]
    async fn deleting_a_realm_announces_a_change() {
        let state = synced_state();
        let mut inner = MockRealmRepository::new();
        inner
            .expect_delete_by_name()
            .returning(|_| Box::pin(async { Ok(()) }));
        let repository = InvalidatingRealmRepository::new(inner, state.clone());

        repository.delete_by_name("acme").await.expect("delete");

        assert_eq!(state.generation(), 1);
    }

    #[tokio::test]
    async fn updating_a_realm_announces_a_change() {
        let state = synced_state();
        let mut inner = MockRealmRepository::new();
        inner.expect_update_realm().returning(|_, name, _| {
            let realm = Realm::new(name);
            Box::pin(async move { Ok(realm) })
        });
        let repository = InvalidatingRealmRepository::new(inner, state.clone());

        repository
            .update_realm("acme".to_string(), "acme2".to_string(), None)
            .await
            .expect("update");

        assert_eq!(state.generation(), 1);
    }

    #[tokio::test]
    async fn updating_realm_settings_announces_a_change() {
        let state = synced_state();
        let mut inner = MockRealmRepository::new();
        inner.expect_update_realm_setting().returning(
            |realm_id,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _,
             _| {
                let settings = RealmSetting::new(realm_id, None);
                Box::pin(async move { Ok(settings) })
            },
        );
        let repository = InvalidatingRealmRepository::new(inner, state.clone());

        repository
            .update_realm_setting(
                RealmId::default(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .expect("update settings");

        assert_eq!(state.generation(), 1);
    }

    #[tokio::test]
    async fn reading_a_realm_does_not_announce_a_change() {
        let state = synced_state();
        let mut inner = MockRealmRepository::new();
        inner
            .expect_get_by_name()
            .returning(|_| Box::pin(async { Ok(None) }));
        let repository = InvalidatingRealmRepository::new(inner, state.clone());

        repository.get_by_name("acme").await.expect("read");

        assert_eq!(state.generation(), 0);
    }
}

use uuid::Uuid;

use crate::auth::Identity;
use crate::common::app_errors::CoreError;
use crate::realm::{LoginAliases, Realm, RealmId, RealmSetting};

pub trait RealmPolicy: Send + Sync {
    fn can_create_realm(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_delete_realm(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_view_realm(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_update_realm(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait RealmRepository: Send + Sync {
    fn fetch_realm(&self) -> impl Future<Output = Result<Vec<Realm>, CoreError>> + Send;

    fn get_by_name(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<Option<Realm>, CoreError>> + Send;

    fn get_by_id(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Option<Realm>, CoreError>> + Send;

    fn create_realm(
        &self,
        name: String,
        display_name: Option<String>,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn update_realm(
        &self,
        realm_name: String,
        name: String,
        display_name: Option<String>,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;
    fn delete_by_name(&self, name: &str) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn create_realm_settings(
        &self,
        realm_id: RealmId,
        algorithm: String,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;

    #[allow(clippy::too_many_arguments)]
    fn update_realm_setting(
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
        seawatch_pii_mode: Option<String>,
        seawatch_pseudo_key: Option<Option<String>>,
        require_mfa: Option<bool>,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;

    fn get_realm_settings(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Option<RealmSetting>, CoreError>> + Send;
}

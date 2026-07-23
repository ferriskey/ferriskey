use std::fmt::Debug;

use uuid::Uuid;

use crate::domain::realm::entities::RealmId;
use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm::entities::{Realm, RealmLoginSetting, RealmSetting, SmtpConfig},
    user::entities::User,
};
use ferriskey_domain::realm::LoginAliases;

pub trait RealmService: Send + Sync {
    fn get_realms_by_user(
        &self,
        identity: Identity,
    ) -> impl Future<Output = Result<Vec<Realm>, CoreError>> + Send;

    fn get_realm_by_name(
        &self,
        identity: Identity,
        input: GetRealmInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn get_realm_setting_by_name(
        &self,
        identity: Identity,
        input: GetRealmSettingInput,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;

    fn create_realm(
        &self,
        identity: Identity,
        input: CreateRealmInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn seed_default_scopes(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn create_realm_with_user(
        &self,
        identity: Identity,
        input: CreateRealmWithUserInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn update_realm(
        &self,
        identity: Identity,
        input: UpdateRealmInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn update_realm_setting(
        &self,
        identity: Identity,
        input: UpdateRealmSettingInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn delete_realm(
        &self,
        identity: Identity,
        input: DeleteRealmInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn get_login_settings(
        &self,
        realm_name: String,
    ) -> impl Future<Output = Result<RealmLoginSetting, CoreError>> + Send;
}

pub trait MailService: Send + Sync {
    fn get_smtp_config(
        &self,
        identity: Identity,
        input: GetSmtpConfigInput,
    ) -> impl Future<Output = Result<SmtpConfig, CoreError>> + Send;

    fn upsert_smtp_config(
        &self,
        identity: Identity,
        input: UpsertSmtpConfigInput,
    ) -> impl Future<Output = Result<SmtpConfig, CoreError>> + Send;

    fn delete_smtp_config(
        &self,
        identity: Identity,
        input: DeleteSmtpConfigInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

// `RealmRepository` and `RealmPolicy` now live in the shared `ferriskey-domain` crate.
// Re-exported so existing `crate::domain::realm::ports::{RealmRepository, RealmPolicy}` sites
// (and `MockRealmRepository`, via the `mock` feature core enables) keep resolving.
// `RealmService` / `MailService` / `SmtpConfigRepository` stay here — they touch
// `RealmLoginSetting` / `SmtpConfig`, which are still `core`-only types.
pub use ferriskey_domain::realm::ports::*;

#[derive(Debug)] // TODO derive debug for instrumetnation
pub struct GetRealmInput {
    pub realm_name: String,
}

pub struct GetRealmSettingInput {
    pub realm_name: String,
}

pub struct CreateRealmInput {
    pub realm_name: String,
    pub display_name: Option<String>,
}

pub struct CreateRealmWithUserInput {
    pub realm_name: String,
    pub user: User,
}

pub struct UpdateRealmInput {
    pub realm_name: String,
    pub name: String,
    pub display_name: Option<String>,
}

pub struct UpdateRealmSettingInput {
    pub realm_name: String,
    pub algorithm: Option<String>,

    pub user_registration_enabled: Option<bool>,
    pub forgot_password_enabled: Option<bool>,
    pub remember_me_enabled: Option<bool>,
    pub magic_link_enabled: Option<bool>,
    pub magic_link_ttl: Option<u32>,
    pub passkey_enabled: Option<bool>,
    pub compass_enabled: Option<bool>,

    pub access_token_lifetime: Option<i64>,
    pub refresh_token_lifetime: Option<i64>,
    pub id_token_lifetime: Option<i64>,
    pub temporary_token_lifetime: Option<i64>,

    pub reset_password_template_id: Option<Option<Uuid>>,
    pub magic_link_template_id: Option<Option<Uuid>>,
    pub email_verification_template_id: Option<Option<Uuid>>,
    pub email_verification_enabled: Option<bool>,
    pub email_verification_ttl_hours: Option<i64>,
    pub lockout_threshold: Option<i32>,
    pub lockout_duration_seconds: Option<i32>,
    pub login_aliases: Option<LoginAliases>,
    pub seawatch_pii_mode: Option<String>,
    pub seawatch_pseudo_key: Option<Option<String>>,
    pub require_mfa: Option<bool>,
}

pub struct DeleteRealmInput {
    pub realm_name: String,
}

#[cfg_attr(test, mockall::automock)]
pub trait SmtpConfigRepository: Send + Sync {
    fn get_by_realm_id(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Option<SmtpConfig>, CoreError>> + Send;

    fn upsert(
        &self,
        config: &SmtpConfig,
    ) -> impl Future<Output = Result<SmtpConfig, CoreError>> + Send;

    fn delete_by_realm_id(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub struct UpsertSmtpConfigInput {
    pub realm_name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub encryption: String,
}

pub struct GetSmtpConfigInput {
    pub realm_name: String,
}

pub struct DeleteSmtpConfigInput {
    pub realm_name: String,
}

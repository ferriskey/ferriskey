use uuid::Uuid;

use crate::domain::{
    abyss::entities::{ldap_provider::LdapProvider, ldap_user::LdapUser},
    common::entities::app_errors::CoreError,
};

#[cfg_attr(test, mockall::automock)]
pub trait LdapRepository: Send + Sync {
    fn create(&self, provider: &LdapProvider)
    -> impl Future<Output = Result<(), CoreError>> + Send;
    fn find_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<LdapProvider>, CoreError>> + Send;
    fn find_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<LdapProvider>, CoreError>> + Send;
    fn update(&self, provider: &LdapProvider)
    -> impl Future<Output = Result<(), CoreError>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait LdapConnector: Send + Sync {
    fn fetch_users(
        &self,
        provider: &LdapProvider,
    ) -> impl Future<Output = Result<Vec<LdapUser>, CoreError>> + Send;
    fn authenticate(
        &self,
        provider: &LdapProvider,
        username: &str,
        password: &str,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

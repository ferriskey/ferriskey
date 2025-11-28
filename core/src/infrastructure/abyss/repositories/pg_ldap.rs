use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::domain::{
    abyss::{entities::ldap_provider::LdapProvider, ports::ldap::LdapRepository},
    common::entities::app_errors::CoreError,
};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct PostgresLdapRepository {
    pub db: DatabaseConnection,
}

impl PostgresLdapRepository {
    #[allow(unused)]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl LdapRepository for PostgresLdapRepository {
    async fn create(&self, _provider: &LdapProvider) -> Result<(), CoreError> {
        Ok(())
    }

    async fn delete(&self, _id: Uuid) -> Result<(), CoreError> {
        todo!()
    }

    async fn find_by_id(&self, _id: Uuid) -> Result<Option<LdapProvider>, CoreError> {
        todo!()
    }

    async fn find_by_realm(&self, _realm_id: Uuid) -> Result<Vec<LdapProvider>, CoreError> {
        todo!()
    }

    async fn update(&self, _provider: &LdapProvider) -> Result<(), CoreError> {
        todo!()
    }
}

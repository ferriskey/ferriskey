use sea_orm::DatabaseConnection;

use crate::domain::{
    abyss::{entities::ldap_provider::LdapProvider, ports::ldap::LdapRepository},
    common::entities::app_errors::CoreError,
};

#[derive(Debug, Clone)]
pub struct PostgresLdapRepository {
    pub db: DatabaseConnection,
}

impl PostgresLdapRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl LdapRepository for PostgresLdapRepository {
    async fn create(&self, provider: &LdapProvider) -> Result<(), CoreError> {
        Ok(())
    }

    async fn delete(&self, id: uuid::Uuid) -> Result<(), CoreError> {
        todo!()
    }

    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<LdapProvider>, CoreError> {
        todo!()
    }

    async fn find_by_realm(&self, realm_id: uuid::Uuid) -> Result<Vec<LdapProvider>, CoreError> {
        todo!()
    }

    async fn update(&self, provider: &LdapProvider) -> Result<(), CoreError> {
        todo!()
    }
}

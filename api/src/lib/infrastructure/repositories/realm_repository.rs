use crate::{
    domain::realm::{
        entities::{error::RealmError, model::Realm},
        ports::RealmRepository,
    },
    infrastructure::db::postgres::Postgres,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PostgresRealmRepository {
    pub postgres: Arc<Postgres>,
}

impl PostgresRealmRepository {
    pub fn new(postgres: Arc<Postgres>) -> Self {
        Self { postgres }
    }
}

impl RealmRepository for PostgresRealmRepository {
    async fn create_realm(&self, name: String) -> Result<Realm, RealmError> {
        let realm = Realm::new(name);

        sqlx::query!(
            r#"
                INSERT INTO realms (id, name, created_at, updated_at)
                VALUES ($1, $2, $3, $4)
            "#,
            realm.id,
            realm.name,
            realm.created_at,
            realm.updated_at
        )
        .execute(&*self.postgres.get_pool())
        .await
        .map_err(|_| RealmError::InternalServerError)?;

        Ok(realm)
    }

    async fn delete_by_name(&self, name: String) -> Result<(), RealmError> {
        sqlx::query!(
            r#"
                DELETE FROM realms WHERE name = $1
            "#,
            name
        )
        .execute(&*self.postgres.get_pool())
        .await
        .map_err(|_| RealmError::InternalServerError)?;
        Ok(())
    }

    async fn get_by_name(&self, name: String) -> Result<Option<Realm>, RealmError> {
        let realm = sqlx::query_as!(Realm, r#"SELECT * FROM realms WHERE name = $1"#, name)
            .fetch_optional(&*self.postgres.get_pool())
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        Ok(realm)
    }
}

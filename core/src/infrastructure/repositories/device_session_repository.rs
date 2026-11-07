use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    prelude::Expr,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::device_session::{
    entities::{DeviceSession, DeviceSessionError},
    ports::DeviceSessionRepository,
};
use crate::domain::realm::entities::{Scoped, Unscoped};
use crate::entity::device_sessions;

impl From<device_sessions::Model> for DeviceSession {
    fn from(model: device_sessions::Model) -> Self {
        DeviceSession {
            id: model.id,
            realm_id: model.realm_id,
            device_token_hash: model.device_token_hash,
            created_at: model.created_at.with_timezone(&Utc),
            last_seen_at: model.last_seen_at.with_timezone(&Utc),
            expires_at: model.expires_at.with_timezone(&Utc),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PostgresDeviceSessionRepository {
    pub db: DatabaseConnection,
}

impl PostgresDeviceSessionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl DeviceSessionRepository for PostgresDeviceSessionRepository {
    async fn create(&self, device: &DeviceSession) -> Result<(), DeviceSessionError> {
        device_sessions::ActiveModel {
            id: Set(device.id),
            realm_id: Set(device.realm_id),
            device_token_hash: Set(device.device_token_hash.clone()),
            created_at: Set(device.created_at.fixed_offset()),
            last_seen_at: Set(device.last_seen_at.fixed_offset()),
            expires_at: Set(device.expires_at.fixed_offset()),
        }
        .insert(&self.db)
        .await
        .map_err(|e| {
            error!("Error creating device session: {:?}", e);
            DeviceSessionError::CreateError
        })?;

        Ok(())
    }

    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Unscoped<DeviceSession>>, DeviceSessionError> {
        let device = device_sessions::Entity::find()
            .filter(device_sessions::Column::DeviceTokenHash.eq(token_hash))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error finding device session by token hash: {:?}", e);
                DeviceSessionError::NotFound
            })?;

        Ok(device.map(|model| Unscoped::new(DeviceSession::from(model))))
    }

    async fn touch_last_seen(
        &self,
        device: &Scoped<DeviceSession>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), DeviceSessionError> {
        let result = device_sessions::Entity::update_many()
            .col_expr(
                device_sessions::Column::LastSeenAt,
                Expr::value(Utc::now().fixed_offset()),
            )
            .col_expr(
                device_sessions::Column::ExpiresAt,
                Expr::value(expires_at.fixed_offset()),
            )
            .filter(device_sessions::Column::Id.eq(device.get().id))
            .filter(device_sessions::Column::RealmId.eq(device.get().realm_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error touching device session: {:?}", e);
                DeviceSessionError::UpdateError
            })?;

        if result.rows_affected == 0 {
            return Err(DeviceSessionError::NotFound);
        }

        Ok(())
    }

    async fn delete_expired(
        &self,
        realm_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<u64, DeviceSessionError> {
        let result = device_sessions::Entity::delete_many()
            .filter(device_sessions::Column::RealmId.eq(realm_id))
            .filter(device_sessions::Column::ExpiresAt.lt(now.fixed_offset()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error deleting expired device sessions: {:?}", e);
                DeviceSessionError::DeleteError
            })?;

        Ok(result.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use sea_orm::Database;
    use sqlx::Executor as _;

    use super::*;
    use crate::domain::realm::entities::{RealmId, RealmScope};
    use crate::domain::session::{entities::UserSession, ports::UserSessionRepository};
    use crate::infrastructure::repositories::user_session_repository::PostgresUserSessionRepository;

    struct Fixture {
        devices: PostgresDeviceSessionRepository,
        sessions: PostgresUserSessionRepository,
        scope: RealmScope,
        user_id: Uuid,
        pool: sqlx::PgPool,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("device_session_repository_test_{}", Uuid::new_v4().simple());

        let admin_pool = sqlx::PgPool::connect(&base_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(r#"CREATE SCHEMA "{schema}""#)))
            .await
            .expect("create test schema");

        let separator = if base_url.contains('?') { '&' } else { '?' };
        let schema_url = format!("{base_url}{separator}options=-c search_path={schema}");
        let pool = sqlx::PgPool::connect(&schema_url)
            .await
            .expect("connect schema pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        let realm_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO realms (id, name) VALUES ($1, $2)")
            .bind(realm_id)
            .bind(format!("realm-{realm_id}"))
            .execute(&pool)
            .await
            .expect("insert realm");
        sqlx::query("INSERT INTO users (id, realm_id, username) VALUES ($1, $2, 'alice')")
            .bind(user_id)
            .bind(realm_id)
            .execute(&pool)
            .await
            .expect("insert user");

        let db = Database::connect(&schema_url)
            .await
            .expect("sea-orm connect");

        let mut realm = crate::domain::realm::entities::Realm::new("test".to_owned());
        realm.id = RealmId::new(realm_id);

        Fixture {
            devices: PostgresDeviceSessionRepository::new(db.clone()),
            sessions: PostgresUserSessionRepository::new(db),
            scope: RealmScope::from_realm(realm),
            user_id,
            pool,
        }
    }

    fn device(fixture: &Fixture, hash: &str, expires_in: Duration) -> DeviceSession {
        let now = Utc::now();
        DeviceSession {
            id: Uuid::new_v4(),
            realm_id: Uuid::from(fixture.scope.id()),
            device_token_hash: hash.to_owned(),
            created_at: now,
            last_seen_at: now,
            expires_at: now + expires_in,
        }
    }

    async fn session(fixture: &Fixture) -> Scoped<UserSession> {
        let session = UserSession::new(
            fixture.user_id,
            Uuid::from(fixture.scope.id()),
            None,
            None,
            Duration::hours(1),
            None,
        );
        fixture
            .sessions
            .create(&session)
            .await
            .expect("create session");
        Unscoped::new(session)
            .in_realm(&fixture.scope)
            .expect("same realm")
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_device_is_found_by_its_token_hash_and_touched() {
        let fixture = setup().await;
        let created = device(&fixture, "hash-a", Duration::hours(1));
        fixture.devices.create(&created).await.expect("create");

        let found = fixture
            .devices
            .find_by_token_hash("hash-a")
            .await
            .expect("find")
            .expect("present")
            .in_realm(&fixture.scope)
            .expect("same realm");
        assert_eq!(found.get().id, created.id);
        assert!(
            fixture
                .devices
                .find_by_token_hash("other")
                .await
                .expect("find")
                .is_none()
        );

        let later = Utc::now() + Duration::days(2);
        fixture
            .devices
            .touch_last_seen(&found, later)
            .await
            .expect("touch");
        let touched = fixture
            .devices
            .find_by_token_hash("hash-a")
            .await
            .expect("find")
            .expect("present")
            .across_realms();
        assert!((touched.expires_at - later).num_milliseconds().abs() < 5);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn expired_devices_are_purged_per_realm_and_detach_their_sessions() {
        let fixture = setup().await;
        let live = device(&fixture, "live", Duration::hours(1));
        let stale = device(&fixture, "stale", -Duration::hours(1));
        fixture.devices.create(&live).await.expect("create live");
        fixture.devices.create(&stale).await.expect("create stale");

        let attached = session(&fixture).await;
        fixture
            .sessions
            .attach_device(&attached, stale.id)
            .await
            .expect("attach");

        let other_realm = fixture
            .devices
            .delete_expired(Uuid::new_v4(), Utc::now())
            .await
            .expect("purge other realm");
        assert_eq!(other_realm, 0);

        let purged = fixture
            .devices
            .delete_expired(stale.realm_id, Utc::now())
            .await
            .expect("purge");
        assert_eq!(purged, 1);
        assert!(
            fixture
                .devices
                .find_by_token_hash("live")
                .await
                .expect("find")
                .is_some()
        );

        let orphan = fixture
            .sessions
            .find_by_id(attached.get().id)
            .await
            .expect("find")
            .expect("the session row outlives its device")
            .across_realms();
        assert_eq!(orphan.device_session_id, None);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn sessions_attach_to_and_detach_from_a_device() {
        let fixture = setup().await;
        let device = device(&fixture, "hash-b", Duration::hours(1));
        fixture.devices.create(&device).await.expect("create");

        let first = session(&fixture).await;
        let second = session(&fixture).await;
        let realm_id = Uuid::from(fixture.scope.id());

        fixture
            .sessions
            .attach_device(&first, device.id)
            .await
            .expect("attach");

        let on_device = fixture
            .sessions
            .find_all_by_device(device.id, realm_id)
            .await
            .expect("list");
        assert_eq!(
            on_device.iter().map(|s| s.id).collect::<Vec<_>>(),
            vec![first.get().id]
        );
        assert!(
            fixture
                .sessions
                .find_all_by_device(device.id, Uuid::new_v4())
                .await
                .expect("list")
                .is_empty()
        );

        fixture
            .sessions
            .detach_device(&first)
            .await
            .expect("detach");
        assert!(
            fixture
                .sessions
                .find_all_by_device(device.id, realm_id)
                .await
                .expect("list")
                .is_empty()
        );
        assert_eq!(second.get().device_session_id, None);

        let vanished = UserSession::new(
            fixture.user_id,
            realm_id,
            None,
            None,
            Duration::hours(1),
            None,
        );
        let vanished = Unscoped::new(vanished)
            .in_realm(&fixture.scope)
            .expect("same realm");
        assert!(
            fixture
                .sessions
                .attach_device(&vanished, device.id)
                .await
                .is_err()
        );
        fixture.pool.close().await;
    }
}

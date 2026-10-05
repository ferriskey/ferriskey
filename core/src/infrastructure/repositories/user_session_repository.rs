use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter, QueryTrait, Select, prelude::Expr,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::common::pagination::{Page, PageRequest};
use crate::domain::realm::entities::{Scoped, Unscoped};
use crate::domain::session::{
    entities::{SessionError, SessionFilter, SessionSortField, UserSession},
    ports::UserSessionRepository,
};
use crate::domain::user::entities::User;
use crate::entity::user_sessions;
use crate::infrastructure::pagination::{SortColumn, contains, paginate, within_naive};

impl SortColumn<user_sessions::Entity> for SessionSortField {
    fn column(&self) -> user_sessions::Column {
        match self {
            SessionSortField::LastSeenAt => user_sessions::Column::LastSeenAt,
            SessionSortField::ExpiresAt => user_sessions::Column::ExpiresAt,
            SessionSortField::CreatedAt => user_sessions::Column::CreatedAt,
        }
    }
}

fn listing_select(
    user_id: Uuid,
    realm_id: Uuid,
    filter: &SessionFilter,
) -> Select<user_sessions::Entity> {
    use user_sessions::Column;

    user_sessions::Entity::find()
        .filter(Column::UserId.eq(user_id))
        .filter(Column::RealmId.eq(realm_id))
        .filter(within_naive(Column::CreatedAt, &filter.created))
        .apply_if(filter.search.as_deref(), |select, value| {
            select.filter(
                Condition::any()
                    .add(contains(Column::IpAddress, value))
                    .add(contains(Column::UserAgent, value)),
            )
        })
        .apply_if(filter.ip_address.as_deref(), |select, value| {
            select.filter(contains(Column::IpAddress, value))
        })
        .apply_if(filter.user_agent.as_deref(), |select, value| {
            select.filter(contains(Column::UserAgent, value))
        })
        .apply_if(filter.persistent, |select, value| {
            select.filter(Column::Persistent.eq(value))
        })
}

impl From<crate::entity::user_sessions::Model> for UserSession {
    fn from(model: crate::entity::user_sessions::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let expires_at = Utc.from_utc_datetime(&model.expires_at);
        let last_seen_at = model.last_seen_at.map(|ref dt| Utc.from_utc_datetime(dt));

        UserSession {
            id: model.id,
            user_id: model.user_id,
            realm_id: model.realm_id,
            user_agent: model.user_agent,
            ip_address: model.ip_address,
            created_at,
            expires_at,
            last_seen_at,
            soft_expiry_duration: None,
            sso_token_hash: model.sso_token_hash,
            persistent: model.persistent,
            authenticated_at: model.authenticated_at.with_timezone(&Utc),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PostgresUserSessionRepository {
    pub db: DatabaseConnection,
}

impl PostgresUserSessionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl UserSessionRepository for PostgresUserSessionRepository {
    async fn create(&self, session: &UserSession) -> Result<(), SessionError> {
        let model = crate::entity::user_sessions::ActiveModel {
            id: Set(session.id),
            user_id: Set(session.user_id),
            realm_id: Set(session.realm_id),
            user_agent: Set(session.user_agent.clone()),
            ip_address: Set(session.ip_address.clone()),
            created_at: Set(session.created_at.naive_utc()),
            expires_at: Set(session.expires_at.naive_utc()),
            last_seen_at: Set(session.last_seen_at.map(|dt| dt.naive_utc())),
            sso_token_hash: Set(session.sso_token_hash.clone()),
            persistent: Set(session.persistent),
            authenticated_at: Set(session.authenticated_at.fixed_offset()),
        };

        model.insert(&self.db).await.map_err(|e| {
            error!("Error creating user session: {:?}", e);
            SessionError::CreateError
        })?;

        Ok(())
    }

    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Unscoped<UserSession>, SessionError> {
        let session = crate::entity::user_sessions::Entity::find()
            .filter(crate::entity::user_sessions::Column::UserId.eq(*user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error finding user session: {:?}", e);
                SessionError::NotFound
            })?
            .ok_or(SessionError::NotFound)?;

        Ok(Unscoped::new(session.into()))
    }

    async fn find_all_by_user_and_realm(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Vec<UserSession>, SessionError> {
        let sessions = crate::entity::user_sessions::Entity::find()
            .filter(crate::entity::user_sessions::Column::UserId.eq(user_id))
            .filter(crate::entity::user_sessions::Column::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Error listing user sessions: {:?}", e);
                SessionError::NotFound
            })?;

        Ok(sessions.into_iter().map(|m| m.into()).collect())
    }

    async fn list(
        &self,
        user: &Scoped<User>,
        request: &PageRequest<SessionFilter, SessionSortField>,
    ) -> Result<Page<UserSession>, SessionError> {
        let user = user.get();
        let select = listing_select(user.id, user.realm_id.into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            error!("Error listing user sessions: {:?}", e);
            SessionError::NotFound
        })?;

        Ok(Page::new(
            models.into_iter().map(UserSession::from).collect(),
            total,
            request.page,
            request.limit,
        ))
    }

    async fn find_by_sso_token_hash(
        &self,
        sso_token_hash: &str,
    ) -> Result<Option<Unscoped<UserSession>>, SessionError> {
        let session = crate::entity::user_sessions::Entity::find()
            .filter(crate::entity::user_sessions::Column::SsoTokenHash.eq(sso_token_hash))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error finding user session by sso token: {:?}", e);
                SessionError::NotFound
            })?;

        Ok(session.map(|m| Unscoped::new(UserSession::from(m))))
    }

    async fn find_by_id(
        &self,
        session_id: Uuid,
    ) -> Result<Option<Unscoped<UserSession>>, SessionError> {
        let session = crate::entity::user_sessions::Entity::find()
            .filter(crate::entity::user_sessions::Column::Id.eq(session_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error finding user session by id: {:?}", e);
                SessionError::NotFound
            })?;

        Ok(session.map(|m| Unscoped::new(m.into())))
    }

    async fn delete(&self, session: &Scoped<UserSession>) -> Result<(), SessionError> {
        crate::entity::user_sessions::Entity::delete_many()
            .filter(crate::entity::user_sessions::Column::Id.eq(session.get().id))
            .filter(crate::entity::user_sessions::Column::RealmId.eq(session.get().realm_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error deleting user session: {:?}", e);
                SessionError::DeleteError
            })?;

        Ok(())
    }

    async fn delete_all_by_user(&self, user_id: Uuid, realm_id: Uuid) -> Result<u64, SessionError> {
        let result = crate::entity::user_sessions::Entity::delete_many()
            .filter(crate::entity::user_sessions::Column::UserId.eq(user_id))
            .filter(crate::entity::user_sessions::Column::RealmId.eq(realm_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error deleting user sessions: {:?}", e);
                SessionError::DeleteError
            })?;

        Ok(result.rows_affected)
    }

    async fn delete_expired_for_user(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, SessionError> {
        let result = crate::entity::user_sessions::Entity::delete_many()
            .filter(crate::entity::user_sessions::Column::UserId.eq(user_id))
            .filter(crate::entity::user_sessions::Column::RealmId.eq(realm_id))
            .filter(crate::entity::user_sessions::Column::ExpiresAt.lt(now.naive_utc()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error deleting expired user sessions: {:?}", e);
                SessionError::DeleteError
            })?;

        Ok(result.rows_affected)
    }

    async fn update_last_seen(&self, session: &Scoped<UserSession>) -> Result<(), SessionError> {
        crate::entity::user_sessions::Entity::update_many()
            .col_expr(
                crate::entity::user_sessions::Column::LastSeenAt,
                Expr::value(Utc::now().naive_utc()),
            )
            .filter(crate::entity::user_sessions::Column::Id.eq(session.get().id))
            .filter(crate::entity::user_sessions::Column::RealmId.eq(session.get().realm_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error updating last_seen_at for user session: {:?}", e);
                SessionError::DeleteError
            })?;

        Ok(())
    }

    async fn set_sso_token_hash(
        &self,
        session: &Scoped<UserSession>,
        sso_token_hash: &str,
    ) -> Result<(), SessionError> {
        crate::entity::user_sessions::Entity::update_many()
            .col_expr(
                crate::entity::user_sessions::Column::SsoTokenHash,
                Expr::value(sso_token_hash),
            )
            .filter(crate::entity::user_sessions::Column::Id.eq(session.get().id))
            .filter(crate::entity::user_sessions::Column::RealmId.eq(session.get().realm_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error setting sso_token_hash for user session: {:?}", e);
                SessionError::UpdateError
            })?;

        Ok(())
    }

    async fn reauthenticate(&self, session: &Scoped<UserSession>) -> Result<(), SessionError> {
        let session = session.get();
        let result = crate::entity::user_sessions::Entity::update_many()
            .col_expr(
                crate::entity::user_sessions::Column::SsoTokenHash,
                Expr::value(session.sso_token_hash.clone()),
            )
            .col_expr(
                crate::entity::user_sessions::Column::AuthenticatedAt,
                Expr::value(session.authenticated_at.fixed_offset()),
            )
            .col_expr(
                crate::entity::user_sessions::Column::ExpiresAt,
                Expr::value(session.expires_at.naive_utc()),
            )
            .col_expr(
                crate::entity::user_sessions::Column::Persistent,
                Expr::value(session.persistent),
            )
            .filter(crate::entity::user_sessions::Column::Id.eq(session.id))
            .filter(crate::entity::user_sessions::Column::RealmId.eq(session.realm_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error re-authenticating a user session: {:?}", e);
                SessionError::UpdateError
            })?;

        // A row revoked meanwhile must not come back to life as a success.
        if result.rows_affected == 0 {
            return Err(SessionError::NotFound);
        }

        Ok(())
    }

    async fn clear_sso_token_hash(
        &self,
        session: &Scoped<UserSession>,
    ) -> Result<(), SessionError> {
        crate::entity::user_sessions::Entity::update_many()
            .col_expr(
                crate::entity::user_sessions::Column::SsoTokenHash,
                Expr::value(Option::<String>::None),
            )
            .filter(crate::entity::user_sessions::Column::Id.eq(session.get().id))
            .filter(crate::entity::user_sessions::Column::RealmId.eq(session.get().realm_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Error clearing sso_token_hash for user session: {:?}", e);
                SessionError::UpdateError
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use chrono::{TimeZone, Utc};

    use super::listing_select;
    use crate::domain::common::pagination::DateRange;
    use crate::domain::session::entities::SessionFilter;

    fn sql(filter: &SessionFilter) -> String {
        listing_select(Uuid::nil(), Uuid::max(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_user_and_its_realm() {
        let sql = sql(&SessionFilter::default());
        assert!(
            sql.contains(r#""user_sessions"."user_id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""user_sessions"."realm_id" = 'ffffffff-ffff-ffff-ffff-ffffffffffff'"#),
            "{sql}"
        );
        assert!(!sql.contains("ILIKE"), "{sql}");
        assert!(!sql.contains(r#""persistent" ="#), "{sql}");
    }

    #[test]
    fn ip_address_and_user_agent_are_escaped_contains_matches() {
        let sql = sql(&SessionFilter {
            ip_address: Some("10.%".to_string()),
            user_agent: Some("fire_".to_string()),
            ..SessionFilter::default()
        });
        assert!(
            sql.contains(r#""user_sessions"."ip_address" ILIKE E'%10.\\%%'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""user_sessions"."user_agent" ILIKE E'%fire\\_%'"#),
            "{sql}"
        );
    }

    #[test]
    fn search_matches_the_ip_address_or_the_user_agent() {
        let sql = sql(&SessionFilter {
            search: Some("a%".to_string()),
            ..SessionFilter::default()
        });
        assert!(
            sql.contains(
                r#"(("user_sessions"."ip_address" ILIKE E'%a\\%%') OR ("user_sessions"."user_agent" ILIKE E'%a\\%%'))"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn created_range_bounds_the_creation_date() {
        let sql = sql(&SessionFilter {
            created: DateRange::new(
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single(),
            ),
            ..SessionFilter::default()
        });
        assert!(
            sql.contains(r#""user_sessions"."created_at" >= '2026-01-01 00:00:00.000000'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""user_sessions"."created_at" < '2026-02-01 00:00:00.000000'"#),
            "{sql}"
        );
    }

    #[test]
    fn persistent_is_an_exact_match() {
        let sql = sql(&SessionFilter {
            persistent: Some(true),
            ..SessionFilter::default()
        });
        assert!(
            sql.contains(r#""user_sessions"."persistent" = TRUE"#),
            "{sql}"
        );
    }
}

use chrono::{DateTime, Utc};
use ferriskey_security::crypto::secret_cipher::SecretCipher;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, sea_query::Expr,
};
use uuid::Uuid;

use crate::domain::{
    common::{entities::app_errors::CoreError, generate_uuid_v7},
    trident::ports::{OtpEnrollment, OtpEnrollmentRepository},
};
use crate::entity::otp_enrollments::{ActiveModel, Column, Entity as OtpEnrollmentEntity, Model};

pub const ENROLLMENT_SECRET_PURPOSE: &str = "otp_enrollments.secret";

#[derive(Debug, Clone)]
pub struct PostgresOtpEnrollmentRepository {
    pub db: DatabaseConnection,
    cipher: SecretCipher,
}

impl PostgresOtpEnrollmentRepository {
    pub fn with_cipher(db: DatabaseConnection, cipher: SecretCipher) -> Self {
        Self { db, cipher }
    }

    fn model_to_domain(&self, model: Model) -> Result<OtpEnrollment, CoreError> {
        let secret = self
            .cipher
            .open(ENROLLMENT_SECRET_PURPOSE, &model.id.to_string(), &model.secret)
            .map_err(|error| {
                tracing::error!(enrollment_id = %model.id, %error, "an OTP enrollment secret could not be opened");
                CoreError::InternalServerError
            })?;

        Ok(OtpEnrollment {
            id: model.id,
            user_id: model.user_id,
            secret,
            expires_at: model.expires_at.and_utc(),
            created_at: model.created_at.and_utc(),
        })
    }
}

impl OtpEnrollmentRepository for PostgresOtpEnrollmentRepository {
    async fn start_enrollment(
        &self,
        user_id: Uuid,
        secret: String,
        expires_at: DateTime<Utc>,
    ) -> Result<OtpEnrollment, CoreError> {
        // Restarting setup invalidates whatever was pending: leaving the previous
        // candidate live would let an abandoned secret stay enrollable until its TTL.
        self.clear_enrollments(user_id).await?;

        let id = generate_uuid_v7();
        let sealed_secret = self
            .cipher
            .seal(ENROLLMENT_SECRET_PURPOSE, &id.to_string(), &secret)
            .map_err(|error| {
                tracing::error!(%error, "an OTP enrollment secret could not be sealed");
                CoreError::InternalServerError
            })?;

        let model = ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
            secret: Set(sealed_secret),
            expires_at: Set(expires_at.naive_utc()),
            consumed_at: Set(None),
            created_at: Set(Utc::now().naive_utc()),
        };

        let model = model.insert(&self.db).await.map_err(|e| {
            tracing::error!(user_id = %user_id, "failed to start OTP enrollment: {e:?}");
            CoreError::InternalServerError
        })?;

        self.model_to_domain(model)
    }

    async fn consume_enrollment(
        &self,
        user_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<OtpEnrollment>, CoreError> {
        let naive_now = now.naive_utc();

        let candidate = OtpEnrollmentEntity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::ConsumedAt.is_null())
            .filter(Column::ExpiresAt.gt(naive_now))
            .order_by_desc(Column::CreatedAt)
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to load OTP enrollment: {e:?}");
                CoreError::InternalServerError
            })?;

        let Some(candidate) = candidate else {
            return Ok(None);
        };

        // Compare-and-swap: the `ConsumedAt.is_null()` predicate is what makes this
        // single-use. A concurrent call that lost the race updates 0 rows and gets
        // `None`, rather than both callers enrolling the same secret.
        let claimed = OtpEnrollmentEntity::update_many()
            .col_expr(Column::ConsumedAt, Expr::value(naive_now))
            .filter(Column::Id.eq(candidate.id))
            .filter(Column::ConsumedAt.is_null())
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to consume OTP enrollment: {e:?}");
                CoreError::InternalServerError
            })?;

        if claimed.rows_affected == 0 {
            return Ok(None);
        }

        self.model_to_domain(candidate).map(Some)
    }

    async fn clear_enrollments(&self, user_id: Uuid) -> Result<u64, CoreError> {
        let deleted = OtpEnrollmentEntity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to clear OTP enrollments: {e:?}");
                CoreError::InternalServerError
            })?;

        Ok(deleted.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;

    const SEALING_KEY: &str = "kid-a:AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";

    async fn setup(prefix: &str) -> (sea_orm::DatabaseConnection, sqlx::PgPool) {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("{prefix}_{}", Uuid::new_v4().simple());

        let admin_pool = sqlx::PgPool::connect(&base_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(r#"CREATE SCHEMA "{}""#, schema)))
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

        let db = SeaOrmDatabase::connect(&schema_url)
            .await
            .expect("sea-orm connect");

        (db, pool)
    }

    async fn insert_user(pool: &sqlx::PgPool) -> Uuid {
        let realm_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(realm_id)
        .bind(format!("realm-{}", realm_id.simple()))
        .execute(pool)
        .await
        .expect("insert realm");

        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, realm_id, username, firstname, lastname, email) \
             VALUES ($1, $2, 'enrolling', '', '', 'enrolling@example.com')",
        )
        .bind(user_id)
        .bind(realm_id)
        .execute(pool)
        .await
        .expect("insert user");

        user_id
    }

    fn sealing(db: DatabaseConnection) -> PostgresOtpEnrollmentRepository {
        PostgresOtpEnrollmentRepository::with_cipher(
            db,
            SecretCipher::from_config(SEALING_KEY).expect("a valid key ring"),
        )
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_pending_secret_is_sealed_at_rest_and_consumed_in_the_clear() {
        let (db, pool) = setup("otp_enrollment_test").await;
        let user_id = insert_user(&pool).await;
        let repository = sealing(db);

        let started = repository
            .start_enrollment(
                user_id,
                "JBSWY3DPEHPK3PXP".to_string(),
                Utc::now() + chrono::Duration::minutes(5),
            )
            .await
            .expect("start the enrollment");
        assert_eq!(started.secret, "JBSWY3DPEHPK3PXP");

        let stored: String = sqlx::query_scalar("SELECT secret FROM otp_enrollments WHERE id = $1")
            .bind(started.id)
            .fetch_one(&pool)
            .await
            .expect("read the stored secret");
        assert!(stored.starts_with("fk1:kid-a:"), "{stored}");
        assert!(!stored.contains("JBSWY3DP"));

        let consumed = repository
            .consume_enrollment(user_id, Utc::now())
            .await
            .expect("consume the enrollment")
            .expect("a pending enrollment");
        assert_eq!(consumed.secret, "JBSWY3DPEHPK3PXP");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_plaintext_pending_secret_is_still_consumed_when_a_cipher_is_configured() {
        let (db, pool) = setup("otp_enrollment_test").await;
        let user_id = insert_user(&pool).await;
        sqlx::query(
            "INSERT INTO otp_enrollments (id, user_id, secret, expires_at) \
             VALUES ($1, $2, 'JBSWY3DPEHPK3PXP', NOW() + interval '5 minutes')",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("insert a legacy enrollment");

        let consumed = sealing(db)
            .consume_enrollment(user_id, Utc::now())
            .await
            .expect("consume the enrollment")
            .expect("a pending enrollment");

        assert_eq!(consumed.secret, "JBSWY3DPEHPK3PXP");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_sealed_pending_secret_is_not_consumable_without_the_key() {
        let (db, pool) = setup("otp_enrollment_test").await;
        let user_id = insert_user(&pool).await;
        sealing(db.clone())
            .start_enrollment(
                user_id,
                "JBSWY3DPEHPK3PXP".to_string(),
                Utc::now() + chrono::Duration::minutes(5),
            )
            .await
            .expect("start the enrollment");

        let result = PostgresOtpEnrollmentRepository::with_cipher(db, SecretCipher::disabled())
            .consume_enrollment(user_id, Utc::now())
            .await;

        assert!(
            !matches!(&result, Ok(Some(enrollment)) if enrollment.secret == "JBSWY3DPEHPK3PXP"),
            "{result:?}"
        );
    }
}

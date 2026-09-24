use chrono::{DateTime, Utc};
use ferriskey_domain::elevation::entities::{Elevation, ElevationId, ElevationProofKind};
use ferriskey_domain::elevation::ports::ElevationRepository;
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::Unscoped;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::common::{entities::app_errors::CoreError, generate_uuid_v7};
use crate::entity::account_elevations::{
    ActiveModel, Column, Entity as AccountElevationEntity, Model,
};

const PROOF_PASSWORD: &str = "password";
const PROOF_OTP: &str = "otp";

#[derive(Debug, Clone)]
pub struct PostgresElevationRepository {
    pub db: DatabaseConnection,
}

impl PostgresElevationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn proof_to_column(proof: ElevationProofKind) -> &'static str {
    match proof {
        ElevationProofKind::Password => PROOF_PASSWORD,
        ElevationProofKind::Otp => PROOF_OTP,
    }
}

fn model_to_domain(model: Model) -> Result<Elevation, CoreError> {
    let proof = match model.proof.as_str() {
        PROOF_PASSWORD => ElevationProofKind::Password,
        PROOF_OTP => ElevationProofKind::Otp,
        other => {
            tracing::error!(
                elevation_id = %model.id,
                proof = %other,
                "an elevation row carries a proof kind this build does not know; refusing it"
            );
            return Err(CoreError::ElevationRequired);
        }
    };

    Ok(Elevation {
        id: ElevationId::new(model.id),
        user_id: model.user_id,
        realm_id: RealmId::new(model.realm_id),
        session_id: model.session_id,
        proof,
        expires_at: model.expires_at.and_utc(),
    })
}

impl ElevationRepository for PostgresElevationRepository {
    async fn start(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
        session_id: Uuid,
        proof: ElevationProofKind,
        expires_at: DateTime<Utc>,
    ) -> Result<Elevation, CoreError> {
        let model = ActiveModel {
            id: Set(generate_uuid_v7()),
            user_id: Set(user_id),
            realm_id: Set(realm_id.into()),
            session_id: Set(session_id),
            proof: Set(proof_to_column(proof).to_string()),
            expires_at: Set(expires_at.naive_utc()),
            created_at: Set(Utc::now().naive_utc()),
        };

        let model = model.insert(&self.db).await.map_err(|e| {
            tracing::error!(user_id = %user_id, "failed to mint an elevation: {e:?}");
            CoreError::InternalServerError
        })?;

        model_to_domain(model)
    }

    async fn find_live(
        &self,
        id: ElevationId,
        user_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<Unscoped<Elevation>>, CoreError> {
        let found = AccountElevationEntity::find()
            .filter(Column::Id.eq(Uuid::from(id)))
            .filter(Column::UserId.eq(user_id))
            .filter(Column::ExpiresAt.gt(now.naive_utc()))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to load an elevation: {e:?}");
                CoreError::InternalServerError
            })?;

        found
            .map(model_to_domain)
            .transpose()
            .map(|elevation| elevation.map(Unscoped::new))
    }

    async fn clear_for_user(&self, user_id: Uuid) -> Result<u64, CoreError> {
        let deleted = AccountElevationEntity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, "failed to clear elevations: {e:?}");
                CoreError::InternalServerError
            })?;

        Ok(deleted.rows_affected)
    }
}

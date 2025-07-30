use chrono::NaiveDateTime;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};
use uuid::Uuid;

use entity::user_required_actions::Entity as UserRequiredActionsEntity;

use crate::domain::{
    common::generate_uuid_v7,
    user::{
        entities::{RequiredAction, RequiredActionError},
        ports::UserRequiredActionRepository,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresUserRequiredActionRepository {
    pub db: DatabaseConnection,
}

impl PostgresUserRequiredActionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl UserRequiredActionRepository for PostgresUserRequiredActionRepository {
    async fn add_required_action(
        &self,
        user_id: Uuid,
        action: RequiredAction,
    ) -> Result<(), RequiredActionError> {
        let created_at: NaiveDateTime = chrono::Utc::now().naive_utc();
        let action_model = entity::user_required_actions::ActiveModel {
            id: Set(generate_uuid_v7()),
            created_at: Set(created_at),
            action: Set(action.to_string()),
            user_id: Set(user_id),
        };

        action_model
            .insert(&self.db)
            .await
            .map_err(|_| RequiredActionError::InternalServerError)?;
        Ok(())
    }

    async fn get_required_actions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RequiredAction>, RequiredActionError> {
        let actions = UserRequiredActionsEntity::find()
            .filter(entity::user_required_actions::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|_| RequiredActionError::InternalServerError)?;

        let actions: Vec<RequiredAction> = actions
            .into_iter()
            .map(|action| {
                action
                    .action
                    .try_into()
                    .map_err(|_| RequiredActionError::InternalServerError)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(actions)
    }

    async fn remove_required_action(
        &self,
        user_id: Uuid,
        action: RequiredAction,
    ) -> Result<(), RequiredActionError> {
        let action = UserRequiredActionsEntity::find()
            .filter(
                entity::user_required_actions::Column::UserId
                    .eq(user_id)
                    .and(entity::user_required_actions::Column::Action.eq(action.to_string())),
            )
            .one(&self.db)
            .await
            .map_err(|_| RequiredActionError::InternalServerError)?
            .ok_or(RequiredActionError::NotFound)?;

        action
            .delete(&self.db)
            .await
            .map_err(|_| RequiredActionError::InternalServerError)?;

        Ok(())
    }

    async fn clear_required_actions(&self, user_id: Uuid) -> Result<u64, RequiredActionError> {
        let rows_affected = UserRequiredActionsEntity::delete_many()
            .filter(entity::user_required_actions::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|_| RequiredActionError::InternalServerError)?
            .rows_affected;

        Ok(rows_affected)
    }
}

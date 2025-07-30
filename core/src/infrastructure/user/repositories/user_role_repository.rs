use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    JoinType, QueryFilter, QuerySelect, RelationTrait, prelude::Expr, sea_query::IntoCondition,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::{
    role::entities::Role,
    user::{entities::UserError, ports::UserRoleRepository},
};

#[derive(Debug, Clone)]
pub struct PostgresUserRoleRepository {
    pub db: DatabaseConnection,
}

impl PostgresUserRoleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl UserRoleRepository for PostgresUserRoleRepository {
    async fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserError> {
        let user_role = entity::user_role::ActiveModel {
            role_id: Set(role_id),
            user_id: Set(user_id),
            ..Default::default()
        };

        user_role
            .insert(&self.db)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Ok(())
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, UserError> {
        let roles = entity::roles::Entity::find()
            .join(
                JoinType::InnerJoin,
                entity::user_role::Relation::Roles
                    .def()
                    .rev()
                    .on_condition(move |_left, right| {
                        Expr::col((right, entity::user_role::Column::UserId))
                            .eq(user_id)
                            .into_condition()
                    }),
            )
            .join(JoinType::LeftJoin, entity::roles::Relation::Clients.def())
            .select_also(entity::clients::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("error getting user roles: {:?}", e);
                UserError::InternalServerError
            })?
            .iter()
            .map(|(model, client)| {
                let mut role: Role = model.clone().into();
                if let Some(client) = client {
                    role.client = Some(client.clone().into());
                }
                role
            })
            .collect::<Vec<Role>>();

        Ok(roles)
    }

    async fn has_role(&self, _user_id: Uuid, _role_id: Uuid) -> Result<bool, UserError> {
        todo!("Implement has_role in PostgresUserRoleRepository");
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserError> {
        let rows = entity::user_role::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(entity::user_role::Column::UserId.eq(user_id))
                    .add(entity::user_role::Column::RoleId.eq(role_id)),
            )
            .exec(&self.db)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        if rows.rows_affected == 0 {
            return Err(UserError::NotFound);
        }

        Ok(())
    }
}

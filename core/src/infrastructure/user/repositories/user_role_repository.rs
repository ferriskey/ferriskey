use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    JoinType, QueryFilter, QuerySelect, RelationTrait, prelude::Expr, sea_query::IntoCondition,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::{
    common::entities::app_errors::CoreError, role::entities::Role, user::ports::UserRoleRepository,
};

#[derive(Debug, Clone)]
pub struct PostgresUserRoleRepository {
    pub db: DatabaseConnection,
}

#[derive(Clone)]
pub enum UserRoleRepoAny {
    Postgres(PostgresUserRoleRepository),
}

impl PostgresUserRoleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl UserRoleRepository for UserRoleRepoAny {
    async fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), CoreError> {
        match self {
            UserRoleRepoAny::Postgres(repo) => repo.assign_role(user_id, role_id).await,
        }
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), CoreError> {
        match self {
            UserRoleRepoAny::Postgres(repo) => repo.revoke_role(user_id, role_id).await,
        }
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, CoreError> {
        match self {
            UserRoleRepoAny::Postgres(repo) => repo.get_user_roles(user_id).await,
        }
    }

    async fn has_role(&self, user_id: Uuid, role_id: Uuid) -> Result<bool, CoreError> {
        match self {
            UserRoleRepoAny::Postgres(repo) => repo.has_role(user_id, role_id).await,
        }
    }
}

impl UserRoleRepository for PostgresUserRoleRepository {
    async fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), CoreError> {
        let user_role = crate::entity::user_role::ActiveModel {
            role_id: Set(role_id),
            user_id: Set(user_id),
            ..Default::default()
        };

        user_role
            .insert(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), CoreError> {
        let rows = crate::entity::user_role::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(crate::entity::user_role::Column::UserId.eq(user_id))
                    .add(crate::entity::user_role::Column::RoleId.eq(role_id)),
            )
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if rows.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, CoreError> {
        let roles = crate::entity::roles::Entity::find()
            .join(
                JoinType::InnerJoin,
                crate::entity::user_role::Relation::Roles
                    .def()
                    .rev()
                    .on_condition(move |_left, right| {
                        Expr::col((right, crate::entity::user_role::Column::UserId))
                            .eq(user_id)
                            .into_condition()
                    }),
            )
            .join(
                JoinType::LeftJoin,
                crate::entity::roles::Relation::Clients.def(),
            )
            .select_also(crate::entity::clients::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("error getting user roles: {:?}", e);
                CoreError::InternalServerError
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

    async fn has_role(&self, _user_id: Uuid, _role_id: Uuid) -> Result<bool, CoreError> {
        todo!("Implement has_role in PostgresUserRoleRepository");
    }
}

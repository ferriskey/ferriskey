use crate::domain::{
    role::entities::models::Role,
    user::{
        dtos::user_dto::{CreateUserDto, UpdateUserDto},
        entities::{error::UserError, model::User, required_action::RequiredAction},
        ports::user_repository::UserRepository,
    },
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    JoinType, ModelTrait, QueryFilter, QuerySelect, RelationTrait, prelude::Expr,
    sea_query::IntoCondition,
};
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pub db: DatabaseConnection,
}

impl PostgresUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl UserRepository for PostgresUserRepository {
    async fn create_user(&self, dto: CreateUserDto) -> Result<User, UserError> {
        let user = User::from_dto(dto);

        let model = entity::users::ActiveModel {
            id: Set(user.id),
            realm_id: Set(user.realm_id),
            username: Set(user.username),
            firstname: Set(user.firstname),
            lastname: Set(user.lastname),
            email: Set(user.email),
            email_verified: Set(user.email_verified),
            enabled: Set(user.enabled),
            client_id: Set(user.client_id),
            created_at: Set(user.created_at.naive_utc()),
            updated_at: Set(user.updated_at.naive_utc()),
        };

        let t = model
            .insert(&self.db)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        let user = t.into();

        Ok(user)
    }

    async fn get_by_username(&self, username: String, realm_id: Uuid) -> Result<User, UserError> {
        let user = entity::users::Entity::find()
            .filter(entity::users::Column::Username.eq(username.clone()))
            .filter(entity::users::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|_| UserError::NotFound)?
            .ok_or(UserError::NotFound)?;

        let user = user.into();

        Ok(user)
    }

    async fn get_by_client_id(&self, client_id: Uuid) -> Result<User, UserError> {
        let user = entity::users::Entity::find()
            .filter(entity::users::Column::ClientId.eq(client_id))
            .one(&self.db)
            .await
            .map_err(|_| UserError::NotFound)?
            .ok_or(UserError::NotFound)?;

        let user = user.into();
        Ok(user)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<User, UserError> {
        let users_model = entity::users::Entity::find()
            .filter(entity::users::Column::Id.eq(id))
            .find_also_related(entity::realms::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Error retrieving user by ID: {:?}", e);
                UserError::NotFound
            })?;

        let user_model = users_model.first().cloned();

        let (user_model, realm_models) = user_model.ok_or(UserError::NotFound)?;
        tracing::info!("user_model: {:?}", user_model);

        let required_actions: Vec<RequiredAction> = user_model
            .find_related(entity::user_required_actions::Entity)
            .all(&self.db)
            .await
            .map_err(|_| UserError::InternalServerError)?
            .into_iter()
            .map(|action| {
                action
                    .action
                    .try_into()
                    .map_err(|_| UserError::InternalServerError)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut user: User = user_model.clone().into();

        user.required_actions = required_actions;

        if let Some(realm_model) = realm_models.as_ref() {
            user.realm = Some(realm_model.clone().into());
        }

        Ok(user)

        // let user_model = entity::users::Entity::find()
        //     .filter(entity::users::Column::Id.eq(id))
        //     .join_as(
        //         JoinType::InnerJoin,
        //         entity::users::Relation::UserRequiredActions.def(),
        //         entity::user_required_actions::Entity,
        //     )
        //     .one(&self.db)
        //     .await
        //     .map_err(|_| UserError::NotFound)?
        //     .ok_or(UserError::NotFound)?;
        // let user: User = user_model.into();
        // Ok(user)
    }

    async fn get_roles_by_user_id(&self, user_id: Uuid) -> Result<Vec<Role>, UserError> {
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

    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Vec<User>, UserError> {
        let users = entity::users::Entity::find()
            .filter(entity::users::Column::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map_err(|_| UserError::NotFound)?;

        let users: Vec<User> = users.into_iter().map(|user| user.into()).collect();

        Ok(users)
    }

    async fn bulk_delete_user(&self, ids: Vec<Uuid>) -> Result<u64, UserError> {
        let rows = entity::users::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(entity::users::Column::Id.is_in(ids.clone()))
                    .add(entity::users::Column::ClientId.is_null()),
            )
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("error deleting users: {:?}", e);
                UserError::NotFound
            })?;

        Ok(rows.rows_affected)
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<u64, UserError> {
        let rows = entity::users::Entity::delete_by_id(user_id)
            .exec(&self.db)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Ok(rows.rows_affected)
    }

    async fn assign_role_to_user(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserError> {
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

    async fn unassign_role_from_user(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserError> {
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

    async fn update_user(&self, user_id: Uuid, dto: UpdateUserDto) -> Result<User, UserError> {
        let user = entity::users::Entity::find()
            .filter(entity::users::Column::Id.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|_| UserError::NotFound)?;

        let user = user.ok_or(UserError::NotFound)?;

        let mut active_model: entity::users::ActiveModel = user.into();

        active_model.firstname = Set(dto.firstname);
        active_model.lastname = Set(dto.lastname);
        active_model.email = Set(dto.email);
        active_model.email_verified = Set(dto.email_verified);
        active_model.enabled = Set(dto.enabled);

        let updated_user = active_model
            .update(&self.db)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Ok(updated_user.into())
    }
}

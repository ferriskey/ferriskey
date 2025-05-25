use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    JoinType, ModelTrait, QueryFilter, QuerySelect, RelationTrait, prelude::Expr,
    sea_query::IntoCondition,
};
use uuid::Uuid;

use crate::domain::{
    realm::entities::realm::Realm,
    role::entities::models::Role,
    user::{
        dtos::user_dto::{CreateUserDto, UpdateUserDto},
        entities::{error::UserError, model::User},
        ports::user_repository::UserRepository,
    },
};
use tracing::error;

impl From<entity::users::Model> for User {
    fn from(model: entity::users::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);

        User {
            id: model.id,
            realm_id: model.realm_id,
            username: model.username,
            firstname: model.firstname,
            lastname: model.lastname,
            email: model.email,
            email_verified: model.email_verified,
            enabled: model.enabled,
            client_id: model.client_id,
            roles: Vec::new(),
            realm: None,
            created_at,
            updated_at,
        }
    }
}

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
        let user_model = entity::users::Entity::find()
            .filter(entity::users::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|_| UserError::NotFound)?
            .ok_or(UserError::NotFound)?;

        let realm = user_model
            .find_related(entity::realms::Entity)
            .one(&self.db)
            .await
            .map_err(|_| UserError::NotFound)?
            .ok_or(UserError::NotFound)?;

        let realm: Realm = realm.into();

        let mut user: User = user_model.into();
        user.realm = Some(realm);

        Ok(user)
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
            .map_err(|_| UserError::NotFound)?;

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

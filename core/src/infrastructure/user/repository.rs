use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, DbErr,
    EntityTrait, ModelTrait, QueryFilter,
};
use tracing::{error, instrument};
use uuid::Uuid;

use crate::domain::realm::entities::RealmId;
use crate::domain::{
    common::entities::app_errors::CoreError,
    user::{
        entities::{RequiredAction, User, UserConfig},
        ports::UserRepository,
        value_objects::{CreateUserRequest, UpdateUserRequest},
    },
};

/// Check if a database error is a unique constraint violation for email
fn is_email_unique_violation(err: &DbErr) -> bool {
    match err {
        DbErr::Query(runtime_err) => {
            let err_str = runtime_err.to_string().to_lowercase();
            // PostgreSQL unique violation error code is 23505
            // The constraint name contains "unique_email_per_realm"
            err_str.contains("unique_email_per_realm")
                || (err_str.contains("23505") && err_str.contains("email"))
                || (err_str.contains("duplicate key") && err_str.contains("email"))
        }
        _ => false,
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
    async fn create_user(&self, dto: CreateUserRequest) -> Result<User, CoreError> {
        let user = User::new(UserConfig {
            client_id: dto.client_id,
            email: dto.email,
            email_verified: dto.email_verified,
            enabled: dto.enabled,
            firstname: dto.firstname,
            lastname: dto.lastname,
            username: dto.username,
            realm_id: dto.realm_id,
        });

        let model = crate::entity::users::ActiveModel {
            id: Set(user.id),
            realm_id: Set(user.realm_id.into()),
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

        let t = model.insert(&self.db).await.map_err(|e| {
            if is_email_unique_violation(&e) {
                CoreError::EmailAlreadyExists
            } else {
                error!("error creating user: {:?}", e);
                CoreError::InternalServerError
            }
        })?;

        let user = t.into();

        Ok(user)
    }

    async fn get_by_username(
        &self,
        username: String,
        realm_id: RealmId,
    ) -> Result<User, CoreError> {
        let users_model = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::Username.eq(username.clone()))
            .filter(crate::entity::users::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .find_also_related(crate::entity::realms::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("error retrieving user by username: {:?}", e);
                CoreError::NotFound
            })?;

        let user_model = users_model.first().cloned();

        let (user_model, realm_model) = user_model.ok_or(CoreError::NotFound)?;

        let required_actions: Vec<RequiredAction> = user_model
            .find_related(crate::entity::user_required_actions::Entity)
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .into_iter()
            .map(|action| {
                action
                    .action
                    .try_into()
                    .map_err(|_| CoreError::InternalServerError)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut user: User = user_model.clone().into();

        user.required_actions = required_actions;

        if let Some(realm_model) = realm_model.as_ref() {
            user.realm = Some(realm_model.clone().into());
        }

        Ok(user)
    }

    #[instrument]
    async fn get_by_client_id(&self, client_id: Uuid) -> Result<User, CoreError> {
        let user = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::ClientId.eq(client_id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::NotFound)?
            .ok_or(CoreError::NotFound)?;

        let user = user.into();
        Ok(user)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<User, CoreError> {
        let users_model = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::Id.eq(id))
            .find_also_related(crate::entity::realms::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Error retrieving user by ID: {:?}", e);
                CoreError::NotFound
            })?;

        let user_model = users_model.first().cloned();

        let (user_model, realm_models) = user_model.ok_or(CoreError::NotFound)?;

        let required_actions: Vec<RequiredAction> = user_model
            .find_related(crate::entity::user_required_actions::Entity)
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .into_iter()
            .map(|action| {
                action
                    .action
                    .try_into()
                    .map_err(|_| CoreError::InternalServerError)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut user: User = user_model.clone().into();

        user.required_actions = required_actions;

        if let Some(realm_model) = realm_models.as_ref() {
            user.realm = Some(realm_model.clone().into());
        }

        Ok(user)
    }

    async fn find_by_realm_id(&self, realm_id: RealmId) -> Result<Vec<User>, CoreError> {
        let users = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::NotFound)?;

        let users: Vec<User> = users.into_iter().map(|user| user.into()).collect();

        Ok(users)
    }

    async fn get_by_email(
        &self,
        email: &str,
        realm_id: RealmId,
    ) -> Result<Option<User>, CoreError> {
        let user = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::Email.eq(email))
            .filter(crate::entity::users::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error retrieving user by email: {:?}", e);
                CoreError::InternalServerError
            })?;

        Ok(user.map(|u| u.into()))
    }

    async fn bulk_delete_user(&self, ids: Vec<Uuid>) -> Result<u64, CoreError> {
        let rows = crate::entity::users::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(crate::entity::users::Column::Id.is_in(ids.clone()))
                    .add(crate::entity::users::Column::ClientId.is_null()),
            )
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("error deleting users: {:?}", e);
                CoreError::NotFound
            })?;

        Ok(rows.rows_affected)
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<u64, CoreError> {
        let rows = crate::entity::users::Entity::delete_by_id(user_id)
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(rows.rows_affected)
    }

    async fn update_user(&self, user_id: Uuid, dto: UpdateUserRequest) -> Result<User, CoreError> {
        let user = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::Id.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::NotFound)?;

        let user = user.ok_or(CoreError::NotFound)?;

        let mut active_model: crate::entity::users::ActiveModel = user.into();

        active_model.firstname = Set(dto.firstname);
        active_model.lastname = Set(dto.lastname);
        active_model.email = Set(dto.email);
        active_model.email_verified = Set(dto.email_verified);
        active_model.enabled = Set(dto.enabled);

        let updated_user = active_model.update(&self.db).await.map_err(|e| {
            if is_email_unique_violation(&e) {
                CoreError::EmailAlreadyExists
            } else {
                error!("error updating user: {:?}", e);
                CoreError::InternalServerError
            }
        })?;

        Ok(updated_user.into())
    }
}

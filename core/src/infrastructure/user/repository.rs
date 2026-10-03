use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, DbErr,
    EntityTrait, ModelTrait, QueryFilter, QuerySelect, QueryTrait, Select, SqlErr,
};
use tracing::{error, instrument};
use uuid::Uuid;

use crate::domain::{
    common::{
        entities::app_errors::CoreError,
        pagination::{Page, PageRequest},
    },
    realm::entities::{RealmId, RealmScope, Scoped, Unscoped},
    user::{
        entities::{RequiredAction, User, UserConfig, UserFilter, UserSortField},
        ports::UserRepository,
        value_objects::{CreateUserRequest, UpdateUserRequest},
    },
};
use crate::entity::{user_role, users};
use crate::infrastructure::pagination::{SortColumn, contains, paginate};

impl SortColumn<users::Entity> for UserSortField {
    fn column(&self) -> users::Column {
        match self {
            UserSortField::Username => users::Column::Username,
            UserSortField::Email => users::Column::Email,
            UserSortField::Firstname => users::Column::Firstname,
            UserSortField::Lastname => users::Column::Lastname,
            UserSortField::Enabled => users::Column::Enabled,
            UserSortField::CreatedAt => users::Column::CreatedAt,
            UserSortField::UpdatedAt => users::Column::UpdatedAt,
        }
    }
}

fn listing_select(realm_id: Uuid, filter: &UserFilter) -> Select<users::Entity> {
    users::Entity::find()
        .filter(users::Column::RealmId.eq(realm_id))
        .apply_if(filter.username.as_deref(), |select, value| {
            select.filter(contains(users::Column::Username, value))
        })
        .apply_if(filter.email.as_deref(), |select, value| {
            select.filter(contains(users::Column::Email, value))
        })
        .apply_if(filter.firstname.as_deref(), |select, value| {
            select.filter(contains(users::Column::Firstname, value))
        })
        .apply_if(filter.lastname.as_deref(), |select, value| {
            select.filter(contains(users::Column::Lastname, value))
        })
        .apply_if(filter.enabled, |select, value| {
            select.filter(users::Column::Enabled.eq(value))
        })
        .apply_if(filter.email_verified, |select, value| {
            select.filter(users::Column::EmailVerified.eq(value))
        })
        .apply_if(filter.service_account, |select, value| {
            select.filter(if value {
                users::Column::ClientId.is_not_null()
            } else {
                users::Column::ClientId.is_null()
            })
        })
        .apply_if(filter.role_id, |select, role_id| {
            select.filter(
                users::Column::Id.in_subquery(
                    user_role::Entity::find()
                        .select_only()
                        .column(user_role::Column::UserId)
                        .filter(user_role::Column::RoleId.eq(role_id))
                        .into_query(),
                ),
            )
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UserUniqueViolation {
    Email,
    Username,
}

fn classify_user_unique_violation_message(message: &str) -> Option<UserUniqueViolation> {
    let err_str = message.to_lowercase();

    if err_str.contains("unique_username_realm_id")
        || err_str.contains("unique_lower_username_per_realm")
        || err_str.contains("key (username")
        || err_str.contains("key (realm_id, lower(username")
        || err_str.contains("username, realm_id")
    {
        Some(UserUniqueViolation::Username)
    } else if err_str.contains("unique_email_per_realm")
        || err_str.contains("unique_lower_email_per_realm")
        || err_str.contains("key (email")
        || err_str.contains("key (realm_id, lower(email")
        || err_str.contains("email, realm_id")
    {
        Some(UserUniqueViolation::Email)
    } else {
        None
    }
}

fn classify_user_unique_violation(err: &DbErr) -> Option<UserUniqueViolation> {
    match err.sql_err()? {
        SqlErr::UniqueConstraintViolation(message) => {
            classify_user_unique_violation_message(&message)
        }
        _ => None,
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
            id: dto.id,
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
            failed_login_attempts: Set(0),
            locked_until: Set(None),
            locale: Set(user.locale),
        };

        let t =
            model
                .insert(&self.db)
                .await
                .map_err(|e| match classify_user_unique_violation(&e) {
                    Some(UserUniqueViolation::Email) => CoreError::EmailAlreadyExists,
                    Some(UserUniqueViolation::Username) => CoreError::UsernameAlreadyExists,
                    None => {
                        error!("error creating user: {:?}", e);
                        CoreError::InternalServerError
                    }
                })?;

        let user = t.into();

        Ok(user)
    }

    async fn find_by_username(
        &self,
        username: String,
        realm_id: RealmId,
    ) -> Result<Option<User>, CoreError> {
        let users_model = crate::entity::users::Entity::find()
            .filter(
                sea_orm::sea_query::Expr::expr(sea_orm::sea_query::Func::lower(
                    sea_orm::sea_query::Expr::col(crate::entity::users::Column::Username),
                ))
                .eq(username.trim().to_lowercase()),
            )
            .filter(crate::entity::users::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .find_also_related(crate::entity::realms::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("error retrieving user by username: {:?}", e);
                CoreError::NotFound
            })?;

        let Some((user_model, realm_model)) = users_model.first().cloned() else {
            return Ok(None);
        };

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

        Ok(Some(user))
    }

    async fn get_by_username(
        &self,
        username: String,
        realm_id: RealmId,
    ) -> Result<User, CoreError> {
        self.find_by_username(username, realm_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    #[instrument]
    async fn get_by_client_id(&self, client_id: Uuid) -> Result<Unscoped<User>, CoreError> {
        let user = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::ClientId.eq(client_id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::NotFound)?
            .ok_or(CoreError::NotFound)?;

        let user = user.into();
        Ok(Unscoped::new(user))
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Unscoped<User>, CoreError> {
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

        Ok(Unscoped::new(user))
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

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<UserFilter, UserSortField>,
    ) -> Result<Page<User>, CoreError> {
        let select = listing_select(scope.id().into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            error!("error listing users: {:?}", e);
            CoreError::InternalServerError
        })?;

        Ok(Page::new(
            models.into_iter().map(User::from).collect(),
            total,
            request.page,
            request.limit,
        ))
    }

    async fn get_by_email(
        &self,
        email: &str,
        realm_id: RealmId,
    ) -> Result<Option<User>, CoreError> {
        let user = crate::entity::users::Entity::find()
            .filter(
                sea_orm::sea_query::Expr::expr(sea_orm::sea_query::Func::lower(
                    sea_orm::sea_query::Expr::col(crate::entity::users::Column::Email),
                ))
                .eq(email.trim().to_lowercase()),
            )
            .filter(crate::entity::users::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Error retrieving user by email: {:?}", e);
                CoreError::InternalServerError
            })?;

        Ok(user.map(|u| u.into()))
    }

    async fn bulk_delete_user(&self, realm_id: RealmId, ids: Vec<Uuid>) -> Result<u64, CoreError> {
        let rows = crate::entity::users::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(crate::entity::users::Column::Id.is_in(ids.clone()))
                    .add(crate::entity::users::Column::ClientId.is_null())
                    .add(crate::entity::users::Column::RealmId.eq(Uuid::from(realm_id))),
            )
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("error deleting users: {:?}", e);
                CoreError::NotFound
            })?;

        Ok(rows.rows_affected)
    }

    async fn delete_user(&self, user: &Scoped<User>) -> Result<u64, CoreError> {
        let rows = crate::entity::users::Entity::delete_by_id(user.get().id)
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(rows.rows_affected)
    }

    async fn update_user(
        &self,
        user: &Scoped<User>,
        dto: UpdateUserRequest,
    ) -> Result<User, CoreError> {
        let user = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::Id.eq(user.get().id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::NotFound)?;

        let user = user.ok_or(CoreError::NotFound)?;

        let mut active_model: crate::entity::users::ActiveModel = user.into();

        if let Some(username) = dto.username {
            active_model.username = Set(username);
        }
        active_model.firstname = Set(dto.firstname);
        active_model.lastname = Set(dto.lastname);
        active_model.email = Set(dto.email);
        active_model.email_verified = Set(dto.email_verified);
        active_model.enabled = Set(dto.enabled);

        let updated_user =
            active_model.update(&self.db).await.map_err(
                |e| match classify_user_unique_violation(&e) {
                    Some(UserUniqueViolation::Email) => CoreError::EmailAlreadyExists,
                    Some(UserUniqueViolation::Username) => CoreError::UsernameAlreadyExists,
                    None => {
                        error!("error updating user: {:?}", e);
                        CoreError::InternalServerError
                    }
                },
            )?;

        Ok(updated_user.into())
    }

    #[instrument(skip(self), err)]
    async fn update_locale(
        &self,
        user: &Scoped<User>,
        locale: Option<String>,
    ) -> Result<User, CoreError> {
        let user = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::Id.eq(user.get().id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("error finding user for locale update: {:?}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active_model: crate::entity::users::ActiveModel = user.into();
        active_model.locale = Set(locale);

        let updated_user = active_model.update(&self.db).await.map_err(|e| {
            error!("error updating user locale: {:?}", e);
            CoreError::InternalServerError
        })?;

        Ok(updated_user.into())
    }

    #[instrument(skip(self), err)]
    async fn increment_failed_login_attempts(
        &self,
        user_id: Uuid,
        locked_until: Option<DateTime<Utc>>,
    ) -> Result<(), CoreError> {
        let user = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::Id.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("error finding user for lockout increment: {:?}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let current_attempts = user.failed_login_attempts;
        let mut active_model: crate::entity::users::ActiveModel = user.into();
        active_model.failed_login_attempts = Set(current_attempts + 1);

        if let Some(until) = locked_until {
            active_model.locked_until = Set(Some(until.fixed_offset()));
            active_model.failed_login_attempts = Set(0);
        }

        active_model.update(&self.db).await.map_err(|e| {
            error!("error incrementing failed login attempts: {:?}", e);
            CoreError::InternalServerError
        })?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn reset_failed_login_attempts(&self, user_id: Uuid) -> Result<(), CoreError> {
        let user = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::Id.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("error finding user for lockout reset: {:?}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active_model: crate::entity::users::ActiveModel = user.into();
        active_model.failed_login_attempts = Set(0);
        active_model.locked_until = Set(None);

        active_model.update(&self.db).await.map_err(|e| {
            error!("error resetting failed login attempts: {:?}", e);
            CoreError::InternalServerError
        })?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn unlock_user(&self, user: &Scoped<User>) -> Result<(), CoreError> {
        let user = crate::entity::users::Entity::find()
            .filter(crate::entity::users::Column::Id.eq(user.get().id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("error finding user for unlock: {:?}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active_model: crate::entity::users::ActiveModel = user.into();
        active_model.failed_login_attempts = Set(0);
        active_model.locked_until = Set(None);

        active_model.update(&self.db).await.map_err(|e| {
            error!("error unlocking user: {:?}", e);
            CoreError::InternalServerError
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::{UserUniqueViolation, classify_user_unique_violation_message, listing_select};
    use crate::domain::user::entities::UserFilter;

    fn sql(filter: &UserFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&UserFilter::default());
        assert!(
            sql.contains(r#""users"."realm_id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
    }

    #[test]
    fn text_filters_are_escaped_contains_matches() {
        let sql = sql(&UserFilter {
            username: Some("a%".to_string()),
            email: Some("b".to_string()),
            firstname: Some("c".to_string()),
            lastname: Some("d".to_string()),
            ..UserFilter::default()
        });
        assert!(
            sql.contains(r#""users"."username" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
        assert!(sql.contains(r#""users"."email" ILIKE '%b%'"#), "{sql}");
        assert!(sql.contains(r#""users"."firstname" ILIKE '%c%'"#), "{sql}");
        assert!(sql.contains(r#""users"."lastname" ILIKE '%d%'"#), "{sql}");
    }

    #[test]
    fn flags_are_exact_matches() {
        let sql = sql(&UserFilter {
            enabled: Some(true),
            email_verified: Some(false),
            ..UserFilter::default()
        });
        assert!(sql.contains(r#""users"."enabled" = TRUE"#), "{sql}");
        assert!(sql.contains(r#""users"."email_verified" = FALSE"#), "{sql}");
    }

    #[test]
    fn service_account_follows_the_client_link() {
        let on = sql(&UserFilter {
            service_account: Some(true),
            ..UserFilter::default()
        });
        assert!(on.contains(r#""users"."client_id" IS NOT NULL"#), "{on}");
        let off = sql(&UserFilter {
            service_account: Some(false),
            ..UserFilter::default()
        });
        assert!(off.contains(r#""users"."client_id" IS NULL"#), "{off}");
    }

    #[test]
    fn role_filter_is_a_subquery_without_join() {
        let sql = sql(&UserFilter {
            role_id: Some(Uuid::nil()),
            ..UserFilter::default()
        });
        assert!(!sql.contains("JOIN"), "{sql}");
        assert!(
            sql.contains(r#""users"."id" IN (SELECT "user_role"."user_id" FROM "user_role" WHERE "user_role"."role_id" = '00000000-0000-0000-0000-000000000000')"#),
            "{sql}"
        );
    }

    #[test]
    fn classifies_username_unique_constraint_violation() {
        let message =
            r#"ERROR: duplicate key value violates unique constraint "unique_username_realm_id""#;

        assert_eq!(
            classify_user_unique_violation_message(message),
            Some(UserUniqueViolation::Username)
        );
    }

    #[test]
    fn classifies_email_unique_constraint_violation() {
        let message =
            r#"ERROR: duplicate key value violates unique constraint "unique_email_per_realm""#;

        assert_eq!(
            classify_user_unique_violation_message(message),
            Some(UserUniqueViolation::Email)
        );
    }

    #[test]
    fn classifies_the_case_insensitive_indexes() {
        assert_eq!(
            classify_user_unique_violation_message(
                r#"ERROR: duplicate key value violates unique constraint "unique_lower_username_per_realm""#
            ),
            Some(UserUniqueViolation::Username)
        );

        assert_eq!(
            classify_user_unique_violation_message(
                r#"ERROR: duplicate key value violates unique constraint "unique_lower_email_per_realm""#
            ),
            Some(UserUniqueViolation::Email)
        );
    }

    #[test]
    fn ignores_unknown_unique_constraint_violation() {
        let message = r#"ERROR: duplicate key value violates unique constraint "users_pkey""#;

        assert_eq!(classify_user_unique_violation_message(message), None);
    }
}

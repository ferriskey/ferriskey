use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait,
    DatabaseConnection, DbBackend, DbErr, EntityTrait, ModelTrait, QueryFilter, QuerySelect,
    QueryTrait, Select, SqlErr, Statement, UpdateMany, sea_query::Expr,
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
use crate::entity::{organization_group_members, organization_members, user_role, users};
use crate::infrastructure::pagination::{SortColumn, contains, paginate, within_naive};

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

fn search(value: &str) -> Condition {
    Condition::any()
        .add(contains(users::Column::Username, value))
        .add(contains(users::Column::Email, value))
        .add(contains(users::Column::Firstname, value))
        .add(contains(users::Column::Lastname, value))
}

fn listing_select(realm_id: Uuid, filter: &UserFilter) -> Select<users::Entity> {
    users::Entity::find()
        .filter(users::Column::RealmId.eq(realm_id))
        .filter(within_naive(users::Column::CreatedAt, &filter.created))
        .apply_if(filter.search.as_deref(), |select, value| {
            select.filter(search(value))
        })
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
        .apply_if(filter.ids.as_deref(), |select, ids| {
            select.filter(users::Column::Id.is_in(ids.iter().copied()))
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
        .apply_if(filter.not_in_group, |select, group_id| {
            select.filter(
                users::Column::Id.not_in_subquery(
                    organization_group_members::Entity::find()
                        .select_only()
                        .column(organization_group_members::Column::UserId)
                        .filter(organization_group_members::Column::GroupId.eq(group_id))
                        .into_query(),
                ),
            )
        })
        .apply_if(filter.not_in_organization, |select, organization_id| {
            select.filter(
                users::Column::Id.not_in_subquery(
                    organization_members::Entity::find()
                        .select_only()
                        .column(organization_members::Column::UserId)
                        .filter(organization_members::Column::OrganizationId.eq(organization_id))
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

fn reset_lockout_update(user_id: Uuid) -> UpdateMany<users::Entity> {
    users::Entity::update_many()
        .col_expr(users::Column::FailedLoginAttempts, Expr::value(0))
        .col_expr(
            users::Column::LockedUntil,
            Expr::value(None::<DateTime<FixedOffset>>),
        )
        .filter(users::Column::Id.eq(user_id))
        .filter(
            Condition::any()
                .add(users::Column::FailedLoginAttempts.ne(0))
                .add(users::Column::LockedUntil.is_not_null()),
        )
}

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pub db: DatabaseConnection,
}

impl PostgresUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn required_actions_of(
        &self,
        user: &users::Model,
    ) -> Result<Vec<RequiredAction>, CoreError> {
        user.find_related(crate::entity::user_required_actions::Entity)
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
            .collect()
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

        let required_actions = self.required_actions_of(&user_model).await?;

        let mut user: User = user_model.into();
        user.required_actions = required_actions;
        if let Some(realm_model) = realm_model {
            user.realm = Some(realm_model.into());
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

        let (user_model, realm_model) =
            users_model.into_iter().next().ok_or(CoreError::NotFound)?;

        let required_actions = self.required_actions_of(&user_model).await?;

        let mut user: User = user_model.into();
        user.required_actions = required_actions;
        if let Some(realm_model) = realm_model {
            user.realm = Some(realm_model.into());
        }

        Ok(Unscoped::new(user))
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
        let user_model = crate::entity::users::Entity::find()
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

        let Some(user_model) = user_model else {
            return Ok(None);
        };

        let required_actions = self.required_actions_of(&user_model).await?;

        let mut user: User = user_model.into();
        user.required_actions = required_actions;

        Ok(Some(user))
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
        threshold: i32,
        locked_until: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let result = self
            .db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE users SET \
                 locked_until = CASE WHEN failed_login_attempts + 1 >= $2 THEN $3 ELSE locked_until END, \
                 failed_login_attempts = CASE WHEN failed_login_attempts + 1 >= $2 THEN 0 ELSE failed_login_attempts + 1 END \
                 WHERE id = $1",
                [
                    user_id.into(),
                    threshold.into(),
                    locked_until.fixed_offset().into(),
                ],
            ))
            .await
            .map_err(|e| {
                error!("error incrementing failed login attempts: {:?}", e);
                CoreError::InternalServerError
            })?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn reset_failed_login_attempts(&self, user_id: Uuid) -> Result<(), CoreError> {
        reset_lockout_update(user_id)
            .exec(&self.db)
            .await
            .map_err(|e| {
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

    use chrono::{TimeZone, Utc};

    use super::{
        UserUniqueViolation, classify_user_unique_violation_message, listing_select,
        reset_lockout_update,
    };
    use crate::domain::common::pagination::DateRange;
    use crate::domain::user::entities::UserFilter;

    fn sql(filter: &UserFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&UserFilter::default());
        assert!(!sql.contains(r#""users"."created_at" >"#), "{sql}");
        assert!(!sql.contains(r#""users"."created_at" <"#), "{sql}");
        assert!(!sql.contains("TRUE"), "{sql}");
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
    fn search_matches_any_of_the_text_fields() {
        let sql = sql(&UserFilter {
            search: Some("a%".to_string()),
            ..UserFilter::default()
        });
        assert!(
            sql.contains(
                r#"(("users"."username" ILIKE E'%a\\%%') OR ("users"."email" ILIKE E'%a\\%%') OR ("users"."firstname" ILIKE E'%a\\%%') OR ("users"."lastname" ILIKE E'%a\\%%'))"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn created_range_bounds_the_creation_date() {
        let sql = sql(&UserFilter {
            created: DateRange::new(
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single(),
            ),
            ..UserFilter::default()
        });
        assert!(
            sql.contains(r#""users"."created_at" >= '2026-01-01 00:00:00.000000'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""users"."created_at" < '2026-02-01 00:00:00.000000'"#),
            "{sql}"
        );
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
    fn ids_filter_is_an_in_list() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let sql = sql(&UserFilter {
            ids: Some(vec![first, second]),
            ..UserFilter::default()
        });
        assert!(
            sql.contains(&format!(r#""users"."id" IN ('{first}', '{second}')"#)),
            "{sql}"
        );
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
    fn not_in_group_excludes_the_members_of_that_group() {
        let group_id = Uuid::from_u128(9);
        let sql = sql(&UserFilter {
            not_in_group: Some(group_id),
            ..UserFilter::default()
        });
        assert!(!sql.contains("JOIN"), "{sql}");
        assert!(
            sql.contains(&format!(
                r#""users"."id" NOT IN (SELECT "organization_group_members"."user_id" FROM "organization_group_members" WHERE "organization_group_members"."group_id" = '{group_id}')"#
            )),
            "{sql}"
        );
        assert!(
            sql.contains(r#""users"."realm_id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
    }

    #[test]
    fn not_in_organization_excludes_the_members_of_that_organization() {
        let organization_id = Uuid::from_u128(11);
        let sql = sql(&UserFilter {
            not_in_organization: Some(organization_id),
            ..UserFilter::default()
        });
        assert!(!sql.contains("JOIN"), "{sql}");
        assert!(
            sql.contains(&format!(
                r#""users"."id" NOT IN (SELECT "organization_members"."user_id" FROM "organization_members" WHERE "organization_members"."organization_id" = '{organization_id}')"#
            )),
            "{sql}"
        );
        assert!(
            sql.contains(r#""users"."realm_id" = '00000000-0000-0000-0000-000000000000'"#),
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

    #[test]
    fn resetting_the_lockout_counter_is_a_single_conditional_update() {
        let sql = reset_lockout_update(Uuid::nil())
            .build(DbBackend::Postgres)
            .to_string();

        assert!(sql.starts_with(r#"UPDATE "users" SET"#), "{sql}");
        assert!(sql.contains(r#""failed_login_attempts" = 0"#), "{sql}");
        assert!(sql.contains(r#""locked_until" = NULL"#), "{sql}");
        assert!(
            sql.contains(r#""id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""failed_login_attempts" <> 0 OR "users"."locked_until" IS NOT NULL"#),
            "{sql}"
        );
    }

    mod postgres {
        use sea_orm::Database as SeaOrmDatabase;
        use sqlx::Executor as _;
        use uuid::Uuid;

        use crate::domain::realm::entities::RealmId;
        use crate::domain::user::entities::RequiredAction;
        use crate::domain::user::ports::UserRepository;
        use crate::infrastructure::user::repository::PostgresUserRepository;

        struct Fixture {
            repository: PostgresUserRepository,
            pool: sqlx::PgPool,
        }

        async fn setup() -> Fixture {
            let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
            });

            let schema = format!("user_repository_test_{}", Uuid::new_v4().simple());

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

            Fixture {
                repository: PostgresUserRepository::new(db),
                pool,
            }
        }

        async fn insert_user(pool: &sqlx::PgPool) -> (Uuid, Uuid) {
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
                 VALUES ($1, $2, 'alice', '', '', 'alice@example.com')",
            )
            .bind(user_id)
            .bind(realm_id)
            .execute(pool)
            .await
            .expect("insert user");

            (realm_id, user_id)
        }

        async fn lockout_state(pool: &sqlx::PgPool, user_id: Uuid) -> (i32, bool, String) {
            sqlx::query_as::<_, (i32, bool, String)>(
                "SELECT failed_login_attempts, locked_until IS NULL, xmin::text FROM users WHERE id = $1",
            )
            .bind(user_id)
            .fetch_one(pool)
            .await
            .expect("read lockout state")
        }

        #[tokio::test]
        #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
        async fn resetting_the_lockout_counter_clears_the_attempts_and_the_lock() {
            let fixture = setup().await;
            let (_, user_id) = insert_user(&fixture.pool).await;
            sqlx::query(
                "UPDATE users SET failed_login_attempts = 3, locked_until = NOW() + INTERVAL '1 hour' WHERE id = $1",
            )
            .bind(user_id)
            .execute(&fixture.pool)
            .await
            .expect("lock the user");

            fixture
                .repository
                .reset_failed_login_attempts(user_id)
                .await
                .expect("reset");

            let (attempts, unlocked, _) = lockout_state(&fixture.pool, user_id).await;
            assert_eq!(attempts, 0);
            assert!(unlocked);
        }

        #[tokio::test]
        #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
        async fn resetting_a_clean_counter_leaves_the_row_untouched() {
            let fixture = setup().await;
            let (_, user_id) = insert_user(&fixture.pool).await;
            let (_, _, before) = lockout_state(&fixture.pool, user_id).await;

            fixture
                .repository
                .reset_failed_login_attempts(user_id)
                .await
                .expect("reset");

            let (attempts, unlocked, after) = lockout_state(&fixture.pool, user_id).await;
            assert_eq!(attempts, 0);
            assert!(unlocked);
            assert_eq!(before, after, "the row version must not change");
        }

        #[tokio::test]
        #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
        async fn looking_a_user_up_by_email_carries_the_required_actions() {
            let fixture = setup().await;
            let (realm_id, user_id) = insert_user(&fixture.pool).await;
            sqlx::query(
                "INSERT INTO user_required_actions (id, user_id, action) VALUES ($1, $2, 'update_password')",
            )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .execute(&fixture.pool)
            .await
            .expect("insert required action");

            let user = fixture
                .repository
                .get_by_email("Alice@Example.com", RealmId::from(realm_id))
                .await
                .expect("lookup")
                .expect("user exists");

            assert_eq!(user.id, user_id);
            assert_eq!(user.required_actions, vec![RequiredAction::UpdatePassword]);
        }
    }
}

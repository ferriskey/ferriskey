use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
#[cfg(test)]
use ferriskey_authz::entities::EntityType;
use ferriskey_authz::entities::{EntityRef, PolicyOrigin};
use ferriskey_authz::tenant::entities::{
    AuthzSchema, AuthzState, Policy, PolicyBody, PolicyTemplate, RelationTuple,
};
use ferriskey_authz::tenant::names::{Namespace, PolicyName, RelationName, SlotId};
use ferriskey_authz::tenant::ports::{
    AuthzSchemaRepository, AuthzStateRepository, PolicyRepository, PolicyTemplateRepository,
    RelationTupleRepository,
};
use ferriskey_domain::common::app_errors::{CoreError, PolicyIssue};
use ferriskey_domain::realm::scope::RealmScope;
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, SqlErr, TransactionTrait,
};
use tracing::error;
use uuid::Uuid;

use crate::entity::{
    authz_policies, authz_policy_templates, authz_realm_states, authz_relation_tuples,
    authz_schemas,
};

/// Stores a realm's tenant authorization rules.
///
/// Every write to the schema, a policy or a template runs in one transaction
/// that first bumps the realm's version, and only if it still holds the
/// version the caller read. The bump takes the row lock, so concurrent writers
/// to the same realm queue up behind it and the loser sees a conflict.
#[derive(Debug, Clone)]
pub struct PostgresAuthzRepository {
    db: DatabaseConnection,
}

impl PostgresAuthzRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn internal(context: &'static str) -> impl FnOnce(DbErr) -> CoreError {
    move |e| {
        error!("{context}: {e}");
        CoreError::InternalServerError
    }
}

fn corrupt(context: &'static str) -> impl FnOnce(CoreError) -> CoreError {
    move |e| {
        error!("{context}: stored value no longer validates: {e}");
        CoreError::InternalServerError
    }
}

fn on_write(e: DbErr) -> CoreError {
    match e.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => CoreError::AlreadyExists,
        _ => {
            error!("authz write failed: {e}");
            CoreError::InternalServerError
        }
    }
}

fn on_policy_write(policy_name: &PolicyName) -> impl Fn(DbErr) -> CoreError + '_ {
    move |e| match e.sql_err() {
        Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
            CoreError::InvalidAuthorizationPolicy(vec![PolicyIssue {
                policy: Some(policy_name.to_string()),
                message: "links a template that does not exist".to_string(),
                line: None,
                column: None,
            }])
        }
        _ => on_write(e),
    }
}

async fn bump_version<C>(conn: &C, realm_id: Uuid, expected_version: i64) -> Result<(), CoreError>
where
    C: ConnectionTrait,
{
    authz_realm_states::Entity::insert(authz_realm_states::ActiveModel {
        realm_id: Set(realm_id),
        enabled: Set(false),
        version: Set(0),
        updated_at: Set(Utc::now().naive_utc()),
    })
    .on_conflict(
        OnConflict::column(authz_realm_states::Column::RealmId)
            .do_nothing()
            .to_owned(),
    )
    .exec_without_returning(conn)
    .await
    .map_err(internal("create authz state"))?;

    let bumped = authz_realm_states::Entity::update_many()
        .col_expr(
            authz_realm_states::Column::Version,
            Expr::col(authz_realm_states::Column::Version).add(1),
        )
        .col_expr(
            authz_realm_states::Column::UpdatedAt,
            Expr::value(Utc::now().naive_utc()),
        )
        .filter(authz_realm_states::Column::RealmId.eq(realm_id))
        .filter(authz_realm_states::Column::Version.eq(expected_version))
        .exec(conn)
        .await
        .map_err(internal("bump authz version"))?;

    if bumped.rows_affected == 0 {
        return Err(CoreError::AuthorizationConflict);
    }
    Ok(())
}

fn origin_to_db(origin: PolicyOrigin) -> &'static str {
    match origin {
        PolicyOrigin::Builtin => "builtin",
        PolicyOrigin::Generated => "generated",
        PolicyOrigin::Custom => "custom",
    }
}

fn origin_from_db(origin: &str) -> Result<PolicyOrigin, CoreError> {
    match origin {
        "builtin" => Ok(PolicyOrigin::Builtin),
        "generated" => Ok(PolicyOrigin::Generated),
        "custom" => Ok(PolicyOrigin::Custom),
        other => {
            error!("unknown authz policy origin {other:?}");
            Err(CoreError::InternalServerError)
        }
    }
}

fn schema_from_db(model: authz_schemas::Model) -> Result<AuthzSchema, CoreError> {
    Ok(AuthzSchema {
        id: model.id,
        namespace: Namespace::try_from(model.namespace).map_err(corrupt("authz namespace"))?,
        source: model.source,
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    })
}

fn template_from_db(model: authz_policy_templates::Model) -> Result<PolicyTemplate, CoreError> {
    Ok(PolicyTemplate {
        id: model.id,
        name: PolicyName::try_from(model.name).map_err(corrupt("authz template name"))?,
        description: model.description,
        source: model.source,
    })
}

fn policy_from_db(model: authz_policies::Model) -> Result<Policy, CoreError> {
    let body = match (model.source, model.template_id, model.template_links) {
        (Some(source), None, None) => PolicyBody::Static { source },
        (None, Some(template_id), Some(links)) => PolicyBody::Linked {
            template_id,
            links: serde_json::from_value::<BTreeMap<SlotId, EntityRef>>(links).map_err(|e| {
                error!("authz policy {}: unreadable template links: {e}", model.id);
                CoreError::InternalServerError
            })?,
        },
        _ => {
            error!("authz policy {} is neither static nor linked", model.id);
            return Err(CoreError::InternalServerError);
        }
    };

    Ok(Policy {
        id: model.id,
        name: PolicyName::try_from(model.name).map_err(corrupt("authz policy name"))?,
        description: model.description,
        origin: origin_from_db(&model.origin)?,
        enabled: model.enabled,
        body,
    })
}

#[cfg(test)]
fn tuple_from_db(model: authz_relation_tuples::Model) -> Result<RelationTuple, CoreError> {
    let subject_relation = match model.subject_relation.as_str() {
        "" => None,
        _ => Some(
            RelationName::try_from(model.subject_relation)
                .map_err(corrupt("authz subject relation"))?,
        ),
    };

    Ok(RelationTuple {
        object: EntityRef::new(EntityType::custom(model.object_type), model.object_id),
        relation: RelationName::try_from(model.relation).map_err(corrupt("authz relation"))?,
        subject: EntityRef::new(EntityType::custom(model.subject_type), model.subject_id),
        subject_relation,
    })
}

fn tuple_key(realm_id: Uuid, tuple: &RelationTuple) -> Condition {
    Condition::all()
        .add(authz_relation_tuples::Column::RealmId.eq(realm_id))
        .add(authz_relation_tuples::Column::ObjectType.eq(tuple.object.ty.as_str()))
        .add(authz_relation_tuples::Column::ObjectId.eq(tuple.object.id.as_str()))
        .add(authz_relation_tuples::Column::Relation.eq(tuple.relation.as_str()))
        .add(authz_relation_tuples::Column::SubjectType.eq(tuple.subject.ty.as_str()))
        .add(authz_relation_tuples::Column::SubjectId.eq(tuple.subject.id.as_str()))
        .add(
            authz_relation_tuples::Column::SubjectRelation.eq(tuple
                .subject_relation
                .as_ref()
                .map_or("", RelationName::as_str)),
        )
}

impl AuthzStateRepository for PostgresAuthzRepository {
    async fn get(&self, realm: &RealmScope) -> Result<AuthzState, CoreError> {
        let model = authz_realm_states::Entity::find_by_id(Uuid::from(realm.id()))
            .one(&self.db)
            .await
            .map_err(internal("read authz state"))?;

        Ok(model.map_or_else(AuthzState::default, |m| AuthzState {
            enabled: m.enabled,
            version: m.version,
        }))
    }

    async fn set_enabled(
        &self,
        realm: &RealmScope,
        enabled: bool,
    ) -> Result<AuthzState, CoreError> {
        let model = authz_realm_states::Entity::insert(authz_realm_states::ActiveModel {
            realm_id: Set(realm.id().into()),
            enabled: Set(enabled),
            version: Set(0),
            updated_at: Set(Utc::now().naive_utc()),
        })
        .on_conflict(
            OnConflict::column(authz_realm_states::Column::RealmId)
                .update_columns([
                    authz_realm_states::Column::Enabled,
                    authz_realm_states::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec_with_returning(&self.db)
        .await
        .map_err(internal("write authz state"))?;

        Ok(AuthzState {
            enabled: model.enabled,
            version: model.version,
        })
    }
}

impl AuthzSchemaRepository for PostgresAuthzRepository {
    async fn get(&self, realm: &RealmScope) -> Result<Option<AuthzSchema>, CoreError> {
        authz_schemas::Entity::find()
            .filter(authz_schemas::Column::RealmId.eq(Uuid::from(realm.id())))
            .one(&self.db)
            .await
            .map_err(internal("read authz schema"))?
            .map(schema_from_db)
            .transpose()
    }

    async fn replace(
        &self,
        realm: &RealmScope,
        schema: AuthzSchema,
        expected_version: i64,
    ) -> Result<AuthzSchema, CoreError> {
        let realm_id = Uuid::from(realm.id());
        let now = schema.updated_at.naive_utc();
        let txn = self.db.begin().await.map_err(internal("begin"))?;

        bump_version(&txn, realm_id, expected_version).await?;

        let model = authz_schemas::Entity::insert(authz_schemas::ActiveModel {
            id: Set(schema.id),
            realm_id: Set(realm_id),
            namespace: Set(schema.namespace.to_string()),
            source: Set(schema.source),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .on_conflict(
            OnConflict::column(authz_schemas::Column::RealmId)
                .update_columns([
                    authz_schemas::Column::Namespace,
                    authz_schemas::Column::Source,
                    authz_schemas::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec_with_returning(&txn)
        .await
        .map_err(on_write)?;

        txn.commit().await.map_err(internal("commit"))?;
        schema_from_db(model)
    }
}

impl PolicyRepository for PostgresAuthzRepository {
    async fn list(&self, realm: &RealmScope) -> Result<Vec<Policy>, CoreError> {
        authz_policies::Entity::find()
            .filter(authz_policies::Column::RealmId.eq(Uuid::from(realm.id())))
            .all(&self.db)
            .await
            .map_err(internal("list authz policies"))?
            .into_iter()
            .map(policy_from_db)
            .collect()
    }

    async fn save(
        &self,
        realm: &RealmScope,
        policy: Policy,
        expected_version: i64,
    ) -> Result<Policy, CoreError> {
        let realm_id = Uuid::from(realm.id());
        let now = Utc::now().naive_utc();
        let (source, template_id, template_links) = match &policy.body {
            PolicyBody::Static { source } => (Some(source.clone()), None, None),
            PolicyBody::Linked { template_id, links } => (
                None,
                Some(*template_id),
                Some(serde_json::to_value(links).map_err(|e| {
                    error!("authz policy {}: template links: {e}", policy.id);
                    CoreError::InternalServerError
                })?),
            ),
        };

        let txn = self.db.begin().await.map_err(internal("begin"))?;
        bump_version(&txn, realm_id, expected_version).await?;

        let updated = authz_policies::Entity::update_many()
            .col_expr(
                authz_policies::Column::Name,
                Expr::value(policy.name.as_str()),
            )
            .col_expr(
                authz_policies::Column::Description,
                Expr::value(policy.description.clone()),
            )
            .col_expr(
                authz_policies::Column::Origin,
                Expr::value(origin_to_db(policy.origin)),
            )
            .col_expr(authz_policies::Column::Enabled, Expr::value(policy.enabled))
            .col_expr(authz_policies::Column::Source, Expr::value(source.clone()))
            .col_expr(authz_policies::Column::TemplateId, Expr::value(template_id))
            .col_expr(
                authz_policies::Column::TemplateLinks,
                Expr::value(template_links.clone()),
            )
            .col_expr(authz_policies::Column::UpdatedAt, Expr::value(now))
            .filter(authz_policies::Column::Id.eq(policy.id))
            .filter(authz_policies::Column::RealmId.eq(realm_id))
            .exec(&txn)
            .await
            .map_err(on_policy_write(&policy.name))?;

        if updated.rows_affected == 0 {
            authz_policies::Entity::insert(authz_policies::ActiveModel {
                id: Set(policy.id),
                realm_id: Set(realm_id),
                name: Set(policy.name.to_string()),
                description: Set(policy.description.clone()),
                origin: Set(origin_to_db(policy.origin).to_string()),
                enabled: Set(policy.enabled),
                source: Set(source),
                template_id: Set(template_id),
                template_links: Set(template_links),
                created_at: Set(now),
                updated_at: Set(now),
            })
            .exec_without_returning(&txn)
            .await
            .map_err(on_policy_write(&policy.name))?;
        }

        txn.commit().await.map_err(internal("commit"))?;
        Ok(policy)
    }

    async fn delete(
        &self,
        realm: &RealmScope,
        id: Uuid,
        expected_version: i64,
    ) -> Result<(), CoreError> {
        let realm_id = Uuid::from(realm.id());
        let txn = self.db.begin().await.map_err(internal("begin"))?;
        bump_version(&txn, realm_id, expected_version).await?;

        let deleted = authz_policies::Entity::delete_many()
            .filter(authz_policies::Column::Id.eq(id))
            .filter(authz_policies::Column::RealmId.eq(realm_id))
            .exec(&txn)
            .await
            .map_err(internal("delete authz policy"))?;
        if deleted.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        txn.commit().await.map_err(internal("commit"))
    }
}

impl PolicyTemplateRepository for PostgresAuthzRepository {
    async fn list(&self, realm: &RealmScope) -> Result<Vec<PolicyTemplate>, CoreError> {
        authz_policy_templates::Entity::find()
            .filter(authz_policy_templates::Column::RealmId.eq(Uuid::from(realm.id())))
            .all(&self.db)
            .await
            .map_err(internal("list authz templates"))?
            .into_iter()
            .map(template_from_db)
            .collect()
    }

    async fn save(
        &self,
        realm: &RealmScope,
        template: PolicyTemplate,
        expected_version: i64,
    ) -> Result<PolicyTemplate, CoreError> {
        let realm_id = Uuid::from(realm.id());
        let now = Utc::now().naive_utc();
        let txn = self.db.begin().await.map_err(internal("begin"))?;
        bump_version(&txn, realm_id, expected_version).await?;

        let updated = authz_policy_templates::Entity::update_many()
            .col_expr(
                authz_policy_templates::Column::Name,
                Expr::value(template.name.as_str()),
            )
            .col_expr(
                authz_policy_templates::Column::Description,
                Expr::value(template.description.clone()),
            )
            .col_expr(
                authz_policy_templates::Column::Source,
                Expr::value(template.source.as_str()),
            )
            .col_expr(authz_policy_templates::Column::UpdatedAt, Expr::value(now))
            .filter(authz_policy_templates::Column::Id.eq(template.id))
            .filter(authz_policy_templates::Column::RealmId.eq(realm_id))
            .exec(&txn)
            .await
            .map_err(on_write)?;

        if updated.rows_affected == 0 {
            authz_policy_templates::Entity::insert(authz_policy_templates::ActiveModel {
                id: Set(template.id),
                realm_id: Set(realm_id),
                name: Set(template.name.to_string()),
                description: Set(template.description.clone()),
                source: Set(template.source.clone()),
                created_at: Set(now),
                updated_at: Set(now),
            })
            .exec_without_returning(&txn)
            .await
            .map_err(on_write)?;
        }

        txn.commit().await.map_err(internal("commit"))?;
        Ok(template)
    }

    async fn delete(
        &self,
        realm: &RealmScope,
        id: Uuid,
        expected_version: i64,
    ) -> Result<(), CoreError> {
        let realm_id = Uuid::from(realm.id());
        let txn = self.db.begin().await.map_err(internal("begin"))?;
        bump_version(&txn, realm_id, expected_version).await?;

        let deleted = authz_policy_templates::Entity::delete_many()
            .filter(authz_policy_templates::Column::Id.eq(id))
            .filter(authz_policy_templates::Column::RealmId.eq(realm_id))
            .exec(&txn)
            .await
            .map_err(|e| match e.sql_err() {
                Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
                    CoreError::AuthorizationTemplateInUse
                }
                _ => {
                    error!("delete authz template: {e}");
                    CoreError::InternalServerError
                }
            })?;
        if deleted.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        txn.commit().await.map_err(internal("commit"))
    }
}

impl RelationTupleRepository for PostgresAuthzRepository {
    async fn write(
        &self,
        realm: &RealmScope,
        writes: Vec<RelationTuple>,
        deletes: Vec<RelationTuple>,
    ) -> Result<(), CoreError> {
        let realm_id = Uuid::from(realm.id());
        let txn = self.db.begin().await.map_err(internal("begin"))?;

        if !deletes.is_empty() {
            let any_of = deletes.iter().fold(Condition::any(), |cond, tuple| {
                cond.add(tuple_key(realm_id, tuple))
            });
            authz_relation_tuples::Entity::delete_many()
                .filter(any_of)
                .exec(&txn)
                .await
                .map_err(internal("delete authz tuples"))?;
        }

        if !writes.is_empty() {
            let now = Utc::now().naive_utc();
            let rows = writes
                .into_iter()
                .map(|tuple| authz_relation_tuples::ActiveModel {
                    realm_id: Set(realm_id),
                    object_type: Set(tuple.object.ty.to_string()),
                    object_id: Set(tuple.object.id),
                    relation: Set(tuple.relation.into()),
                    subject_type: Set(tuple.subject.ty.to_string()),
                    subject_id: Set(tuple.subject.id),
                    subject_relation: Set(tuple
                        .subject_relation
                        .map(String::from)
                        .unwrap_or_default()),
                    created_at: Set(now),
                });
            authz_relation_tuples::Entity::insert_many(rows)
                .on_conflict(
                    OnConflict::columns([
                        authz_relation_tuples::Column::RealmId,
                        authz_relation_tuples::Column::ObjectType,
                        authz_relation_tuples::Column::ObjectId,
                        authz_relation_tuples::Column::Relation,
                        authz_relation_tuples::Column::SubjectType,
                        authz_relation_tuples::Column::SubjectId,
                        authz_relation_tuples::Column::SubjectRelation,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec_without_returning(&txn)
                .await
                .map_err(internal("write authz tuples"))?;
        }

        txn.commit().await.map_err(internal("commit"))
    }

    async fn delete_by_subject(
        &self,
        realm: &RealmScope,
        subject: &EntityRef,
    ) -> Result<u64, CoreError> {
        let deleted = authz_relation_tuples::Entity::delete_many()
            .filter(authz_relation_tuples::Column::RealmId.eq(Uuid::from(realm.id())))
            .filter(authz_relation_tuples::Column::SubjectType.eq(subject.ty.as_str()))
            .filter(authz_relation_tuples::Column::SubjectId.eq(subject.id.as_str()))
            .exec(&self.db)
            .await
            .map_err(internal("delete authz tuples by subject"))?;

        Ok(deleted.rows_affected)
    }
}

#[cfg(test)]
impl PostgresAuthzRepository {
    /// Every tuple of the realm. Only tests read tuples back until relation
    /// expansion lands; the port exposes no read yet.
    pub async fn list_tuples(&self, realm: &RealmScope) -> Result<Vec<RelationTuple>, CoreError> {
        authz_relation_tuples::Entity::find()
            .filter(authz_relation_tuples::Column::RealmId.eq(Uuid::from(realm.id())))
            .all(&self.db)
            .await
            .map_err(internal("list authz tuples"))?
            .into_iter()
            .map(tuple_from_db)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use ferriskey_domain::realm::{Realm, RealmId};
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;

    use super::*;

    struct Fixture {
        repository: PostgresAuthzRepository,
        realm_a: RealmScope,
        realm_b: RealmScope,
        pool: sqlx::PgPool,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("authz_repository_test_{}", Uuid::new_v4().simple());

        let admin_pool = sqlx::PgPool::connect(&base_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(r#"CREATE SCHEMA "{schema}""#)))
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

        let mut scopes = Vec::new();
        for label in ["tenant-a", "tenant-b"] {
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
            )
            .bind(id)
            .bind(format!("{label}-{id}"))
            .execute(&pool)
            .await
            .expect("insert test realm");

            let mut realm = Realm::new(label.to_string());
            realm.id = RealmId::from(id);
            scopes.push(RealmScope::from_realm(realm));
        }
        let realm_b = scopes.pop().expect("two realms");
        let realm_a = scopes.pop().expect("two realms");

        let db = SeaOrmDatabase::connect(&schema_url)
            .await
            .expect("sea-orm connect");

        Fixture {
            repository: PostgresAuthzRepository::new(db),
            realm_a,
            realm_b,
            pool,
        }
    }

    fn name(value: &str) -> PolicyName {
        PolicyName::try_from(value.to_string()).expect("valid name")
    }

    fn static_policy(policy_name: &str) -> Policy {
        Policy {
            id: Uuid::new_v4(),
            name: name(policy_name),
            description: None,
            origin: PolicyOrigin::Custom,
            enabled: true,
            body: PolicyBody::Static {
                source: "permit(principal, action, resource);".to_string(),
            },
        }
    }

    fn template(template_name: &str) -> PolicyTemplate {
        PolicyTemplate {
            id: Uuid::new_v4(),
            name: name(template_name),
            description: Some("owners".to_string()),
            source: "permit(principal == ?principal, action, resource);".to_string(),
        }
    }

    fn linked_to(template: &PolicyTemplate, policy_name: &str) -> Policy {
        Policy {
            id: Uuid::new_v4(),
            name: name(policy_name),
            description: None,
            origin: PolicyOrigin::Generated,
            enabled: true,
            body: PolicyBody::Linked {
                template_id: template.id,
                links: BTreeMap::from([(
                    SlotId::Principal,
                    EntityRef::new(
                        EntityType::custom("FerrisKey::Role"),
                        Uuid::new_v4().to_string(),
                    ),
                )]),
            },
        }
    }

    fn tuple(subject_type: &'static str, subject_id: Uuid) -> RelationTuple {
        RelationTuple {
            object: EntityRef::new(EntityType::custom("App::Document"), "42"),
            relation: RelationName::try_from("viewer".to_string()).expect("valid relation"),
            subject: EntityRef::new(EntityType::custom(subject_type), subject_id.to_string()),
            subject_relation: None,
        }
    }

    async fn version(f: &Fixture, realm: &RealmScope) -> i64 {
        AuthzStateRepository::get(&f.repository, realm)
            .await
            .expect("read state")
            .version
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn every_write_bumps_the_version_exactly_once() {
        let f = setup().await;
        let realm = &f.realm_a;
        assert_eq!(version(&f, realm).await, 0);

        let tpl = template("owner-reads");
        PolicyTemplateRepository::save(&f.repository, realm, tpl.clone(), 0)
            .await
            .expect("save template");
        assert_eq!(version(&f, realm).await, 1);

        let policy = linked_to(&tpl, "alice-reads");
        PolicyRepository::save(&f.repository, realm, policy.clone(), 1)
            .await
            .expect("save policy");
        assert_eq!(version(&f, realm).await, 2);

        let schema = AuthzSchema {
            id: Uuid::new_v4(),
            namespace: Namespace::default(),
            source: "entity Document;".to_string(),
            updated_at: Utc::now(),
        };
        AuthzSchemaRepository::replace(&f.repository, realm, schema, 2)
            .await
            .expect("replace schema");
        assert_eq!(version(&f, realm).await, 3);

        PolicyRepository::delete(&f.repository, realm, policy.id, 3)
            .await
            .expect("delete policy");
        assert_eq!(version(&f, realm).await, 4);

        AuthzStateRepository::set_enabled(&f.repository, realm, true)
            .await
            .expect("enable");
        assert_eq!(version(&f, realm).await, 4, "enabling changes no rule");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_write_at_a_stale_version_is_refused_and_changes_nothing() {
        let f = setup().await;
        let realm = &f.realm_a;
        PolicyRepository::save(&f.repository, realm, static_policy("first"), 0)
            .await
            .expect("first write");

        let stale = PolicyRepository::save(&f.repository, realm, static_policy("second"), 0).await;

        assert!(matches!(stale, Err(CoreError::AuthorizationConflict)));
        assert_eq!(version(&f, realm).await, 1);
        let names: Vec<String> = PolicyRepository::list(&f.repository, realm)
            .await
            .expect("list")
            .into_iter()
            .map(|p| p.name.to_string())
            .collect();
        assert_eq!(names, vec!["first".to_string()]);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_failed_write_rolls_the_version_back() {
        let f = setup().await;
        let realm = &f.realm_a;
        PolicyRepository::save(&f.repository, realm, static_policy("taken"), 0)
            .await
            .expect("first write");

        let duplicate =
            PolicyRepository::save(&f.repository, realm, static_policy("taken"), 1).await;

        assert!(matches!(duplicate, Err(CoreError::AlreadyExists)));
        assert_eq!(version(&f, realm).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_policy_round_trips_with_its_template_links() {
        let f = setup().await;
        let realm = &f.realm_a;
        let tpl = template("owner-reads");
        PolicyTemplateRepository::save(&f.repository, realm, tpl.clone(), 0)
            .await
            .expect("save template");
        let policy = linked_to(&tpl, "alice-reads");
        PolicyRepository::save(&f.repository, realm, policy.clone(), 1)
            .await
            .expect("save policy");

        let stored = PolicyRepository::list(&f.repository, realm)
            .await
            .expect("list");
        assert_eq!(stored, vec![policy]);
        let templates = PolicyTemplateRepository::list(&f.repository, realm)
            .await
            .expect("list templates");
        assert_eq!(templates, vec![tpl]);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_policy_linking_another_realms_template_is_refused_and_changes_nothing() {
        let f = setup().await;
        let tpl = template("owner-reads");
        PolicyTemplateRepository::save(&f.repository, &f.realm_b, tpl.clone(), 0)
            .await
            .expect("save template");

        let result =
            PolicyRepository::save(&f.repository, &f.realm_a, linked_to(&tpl, "alice-reads"), 0)
                .await;

        assert!(
            matches!(result, Err(CoreError::InvalidAuthorizationPolicy(_))),
            "got {result:?}"
        );
        assert_eq!(version(&f, &f.realm_a).await, 0);
        assert!(
            PolicyRepository::list(&f.repository, &f.realm_a)
                .await
                .expect("list")
                .is_empty()
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_template_linked_by_a_policy_cannot_be_deleted() {
        let f = setup().await;
        let realm = &f.realm_a;
        let tpl = template("owner-reads");
        PolicyTemplateRepository::save(&f.repository, realm, tpl.clone(), 0)
            .await
            .expect("save template");
        PolicyRepository::save(&f.repository, realm, linked_to(&tpl, "alice-reads"), 1)
            .await
            .expect("save policy");

        let result = PolicyTemplateRepository::delete(&f.repository, realm, tpl.id, 2).await;

        assert!(
            matches!(result, Err(CoreError::AuthorizationTemplateInUse)),
            "got {result:?}"
        );
        assert_eq!(version(&f, realm).await, 2);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_realm_sees_and_touches_none_of_another_realms_rules() {
        let f = setup().await;
        let tpl = template("owner-reads");
        PolicyTemplateRepository::save(&f.repository, &f.realm_a, tpl.clone(), 0)
            .await
            .expect("save template in A");
        let policy = static_policy("a-only");
        PolicyRepository::save(&f.repository, &f.realm_a, policy.clone(), 1)
            .await
            .expect("save policy in A");

        assert!(
            PolicyRepository::list(&f.repository, &f.realm_b)
                .await
                .expect("list B")
                .is_empty()
        );
        assert!(
            PolicyTemplateRepository::list(&f.repository, &f.realm_b)
                .await
                .expect("list B")
                .is_empty()
        );
        assert!(
            AuthzSchemaRepository::get(&f.repository, &f.realm_b)
                .await
                .expect("schema B")
                .is_none()
        );

        let delete_from_b = PolicyRepository::delete(&f.repository, &f.realm_b, policy.id, 0).await;
        assert!(matches!(delete_from_b, Err(CoreError::NotFound)));

        let link_from_b =
            PolicyRepository::save(&f.repository, &f.realm_b, linked_to(&tpl, "b-steals"), 0).await;
        assert!(link_from_b.is_err(), "B linked a template of A");

        let mut renamed = policy.clone();
        renamed.name = name("hijacked");
        PolicyRepository::save(&f.repository, &f.realm_b, renamed, 0)
            .await
            .expect_err("B overwrote a policy of A");
        assert_eq!(
            PolicyRepository::list(&f.repository, &f.realm_a)
                .await
                .expect("list A"),
            vec![policy]
        );

        let user = Uuid::new_v4();
        RelationTupleRepository::write(
            &f.repository,
            &f.realm_a,
            vec![tuple("FerrisKey::User", user)],
            vec![],
        )
        .await
        .expect("write tuple in A");
        let removed = RelationTupleRepository::delete_by_subject(
            &f.repository,
            &f.realm_b,
            &EntityRef::new(EntityType::custom("FerrisKey::User"), user.to_string()),
        )
        .await
        .expect("delete in B");
        assert_eq!(removed, 0);
        assert_eq!(
            f.repository
                .list_tuples(&f.realm_a)
                .await
                .expect("tuples A")
                .len(),
            1
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn tuple_writes_and_deletes_apply_together() {
        let f = setup().await;
        let realm = &f.realm_a;
        let (alice, bob) = (Uuid::new_v4(), Uuid::new_v4());

        RelationTupleRepository::write(
            &f.repository,
            realm,
            vec![tuple("FerrisKey::User", alice)],
            vec![],
        )
        .await
        .expect("first write");
        RelationTupleRepository::write(
            &f.repository,
            realm,
            vec![tuple("FerrisKey::User", bob), tuple("FerrisKey::User", bob)],
            vec![tuple("FerrisKey::User", alice)],
        )
        .await
        .expect("swap");

        assert_eq!(
            f.repository.list_tuples(realm).await.expect("tuples"),
            vec![tuple("FerrisKey::User", bob)]
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn deleting_a_user_forgets_its_tuples() {
        let f = setup().await;
        let realm = &f.realm_a;
        let user = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, realm_id, username) VALUES ($1, $2, 'alice')")
            .bind(user)
            .bind(Uuid::from(realm.id()))
            .execute(&f.pool)
            .await
            .expect("insert user");
        let mut owned = tuple("App::Folder", Uuid::new_v4());
        owned.object = EntityRef::new(EntityType::custom("FerrisKey::User"), user.to_string());
        RelationTupleRepository::write(
            &f.repository,
            realm,
            vec![tuple("FerrisKey::User", user), owned],
            vec![],
        )
        .await
        .expect("write tuples");

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user)
            .execute(&f.pool)
            .await
            .expect("delete user");

        assert!(
            f.repository
                .list_tuples(realm)
                .await
                .expect("tuples")
                .is_empty()
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn deleting_an_organization_forgets_the_tuples_of_its_cascaded_groups() {
        let f = setup().await;
        let realm = &f.realm_a;
        let (org, parent, child) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        sqlx::query(
            "INSERT INTO organizations (id, realm_id, name, alias) VALUES ($1, $2, 'acme', 'acme')",
        )
        .bind(org)
        .bind(Uuid::from(realm.id()))
        .execute(&f.pool)
        .await
        .expect("insert organization");
        for (id, parent_id) in [(parent, None), (child, Some(parent))] {
            sqlx::query(
                "INSERT INTO organization_groups (id, organization_id, parent_group_id, name) VALUES ($1, $2, $3, $4)",
            )
            .bind(id)
            .bind(org)
            .bind(parent_id)
            .bind(id.to_string())
            .execute(&f.pool)
            .await
            .expect("insert group");
        }
        RelationTupleRepository::write(
            &f.repository,
            realm,
            vec![
                tuple("FerrisKey::Organization", org),
                tuple("FerrisKey::Group", parent),
                tuple("FerrisKey::Group", child),
            ],
            vec![],
        )
        .await
        .expect("write tuples");

        sqlx::query("DELETE FROM organizations WHERE id = $1")
            .bind(org)
            .execute(&f.pool)
            .await
            .expect("delete organization");

        assert!(
            f.repository
                .list_tuples(realm)
                .await
                .expect("tuples")
                .is_empty()
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_policy_row_must_be_static_or_linked_never_both_or_neither() {
        let f = setup().await;
        let realm_id = Uuid::from(f.realm_a.id());
        let tpl = template("owner-reads");
        PolicyTemplateRepository::save(&f.repository, &f.realm_a, tpl.clone(), 0)
            .await
            .expect("save template");

        for (source, template_id, links) in [
            (
                Some("permit(principal, action, resource);"),
                Some(tpl.id),
                Some("{}"),
            ),
            (None, None, None),
        ] {
            let result = sqlx::query(
                "INSERT INTO authz_policies (id, realm_id, name, origin, enabled, source, template_id, template_links)
                 VALUES ($1, $2, $3, 'custom', true, $4, $5, $6::jsonb)",
            )
            .bind(Uuid::new_v4())
            .bind(realm_id)
            .bind(Uuid::new_v4().to_string())
            .bind(source)
            .bind(template_id)
            .bind(links)
            .execute(&f.pool)
            .await;

            assert!(
                result.is_err(),
                "accepted source={source:?} template={template_id:?}"
            );
        }
    }
}

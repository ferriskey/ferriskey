use std::collections::{BTreeMap, BTreeSet};

use ferriskey_authz::entities::EntityRef;
use ferriskey_authz::tenant::evaluation::{
    ClientFacts, GroupFacts, OrganizationFacts, PrincipalFacts, PrincipalSlice, RoleFacts,
    UserFacts,
};
use ferriskey_authz::tenant::ports::PrincipalSliceRepository;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::scope::RealmScope;
use sea_orm::sea_query::Query;
use sea_orm::{
    ColumnTrait, DatabaseBackend, DatabaseConnection, DbErr, EntityTrait, FromQueryResult,
    QueryFilter, Statement,
};
use tracing::error;
use uuid::Uuid;

use crate::entity::{
    clients, organization_members, organizations, roles, user_attributes, user_role, users,
};

const USER_TYPE: &str = "FerrisKey::User";
const CLIENT_TYPE: &str = "FerrisKey::Client";

const GROUPS_WITH_ANCESTORS: &str = r#"
WITH RECURSIVE chain (id, organization_id, parent_group_id, name, direct) AS (
    SELECT g.id, g.organization_id, g.parent_group_id, g.name, TRUE
    FROM organization_group_members m
    JOIN organization_groups g ON g.id = m.group_id
    JOIN organizations o ON o.id = g.organization_id
    WHERE m.user_id = $1 AND o.realm_id = $2
    UNION
    SELECT p.id, p.organization_id, p.parent_group_id, p.name, FALSE
    FROM chain c
    JOIN organization_groups p ON p.id = c.parent_group_id
    JOIN organizations o ON o.id = p.organization_id
    WHERE o.realm_id = $2
)
SELECT id, organization_id, parent_group_id, name, direct FROM chain
"#;

#[derive(Debug, Clone)]
pub struct PostgresPrincipalSliceRepository {
    db: DatabaseConnection,
}

impl PostgresPrincipalSliceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[derive(Debug, FromQueryResult)]
struct GroupRow {
    id: Uuid,
    organization_id: Uuid,
    parent_group_id: Option<Uuid>,
    name: String,
    direct: bool,
}

fn internal(context: &'static str) -> impl FnOnce(DbErr) -> CoreError {
    move |e| {
        error!("{context}: {e}");
        CoreError::InternalServerError
    }
}

impl PostgresPrincipalSliceRepository {
    async fn roles_of_user(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<roles::Model>, CoreError> {
        roles::Entity::find()
            .filter(roles::Column::RealmId.eq(realm_id))
            .filter(
                roles::Column::Id.in_subquery(
                    Query::select()
                        .column(user_role::Column::RoleId)
                        .from(user_role::Entity)
                        .and_where(user_role::Column::UserId.eq(user_id))
                        .to_owned(),
                ),
            )
            .all(&self.db)
            .await
            .map_err(internal("load principal roles"))
    }

    async fn load_user(
        &self,
        realm_id: Uuid,
        user: users::Model,
    ) -> Result<PrincipalSlice, CoreError> {
        let attributes = user_attributes::Entity::find()
            .filter(user_attributes::Column::UserId.eq(user.id))
            .filter(user_attributes::Column::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map_err(internal("load principal attributes"))?;
        let roles = self.roles_of_user(realm_id, user.id).await?;
        let direct_organizations = organization_members::Entity::find()
            .filter(organization_members::Column::UserId.eq(user.id))
            .all(&self.db)
            .await
            .map_err(internal("load principal organizations"))?;
        let group_rows = GroupRow::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            GROUPS_WITH_ANCESTORS,
            [user.id.into(), realm_id.into()],
        ))
        .all(&self.db)
        .await
        .map_err(internal("load principal groups"))?;

        let group_ids: Vec<Uuid> = group_rows
            .iter()
            .filter(|row| row.direct)
            .map(|row| row.id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let groups: Vec<GroupFacts> = group_rows
            .iter()
            .map(|row| (row.id, row))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .map(|row| GroupFacts {
                id: row.id,
                name: row.name.clone(),
                parent_id: row.parent_group_id,
                organization_id: row.organization_id,
            })
            .collect();

        let wanted: BTreeSet<Uuid> = direct_organizations
            .iter()
            .map(|m| m.organization_id)
            .chain(groups.iter().map(|g| g.organization_id))
            .collect();
        let organizations = organizations::Entity::find()
            .filter(organizations::Column::RealmId.eq(realm_id))
            .filter(organizations::Column::Id.is_in(wanted))
            .all(&self.db)
            .await
            .map_err(internal("load principal organization facts"))?;
        let known: BTreeSet<Uuid> = organizations.iter().map(|o| o.id).collect();
        let organization_ids = direct_organizations
            .iter()
            .map(|m| m.organization_id)
            .filter(|id| known.contains(id))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        Ok(PrincipalSlice {
            principal: PrincipalFacts::User(UserFacts {
                id: user.id,
                username: user.username,
                email: user.email,
                email_verified: user.email_verified,
                enabled: user.enabled,
                tags: attributes.into_iter().map(|a| (a.key, a.value)).collect(),
                role_ids: roles.iter().map(|r| r.id).collect(),
                group_ids,
                organization_ids,
            }),
            roles: role_facts(roles),
            groups,
            organizations: organizations
                .into_iter()
                .map(|o| OrganizationFacts {
                    id: o.id,
                    alias: o.alias,
                    name: o.name,
                })
                .collect(),
        })
    }

    async fn load_client(
        &self,
        realm_id: Uuid,
        client: clients::Model,
    ) -> Result<Option<PrincipalSlice>, CoreError> {
        if !client.enabled || !client.service_account_enabled {
            return Ok(None);
        }
        let account = users::Entity::find()
            .filter(users::Column::ClientId.eq(client.id))
            .filter(users::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(internal("load service account"))?;
        let Some(account) = account.filter(|account| account.enabled) else {
            return Ok(None);
        };
        let roles = self.roles_of_user(realm_id, account.id).await?;

        Ok(Some(PrincipalSlice {
            principal: PrincipalFacts::Client(ClientFacts {
                id: client.id,
                client_id: client.client_id,
                role_ids: roles.iter().map(|r| r.id).collect(),
            }),
            roles: role_facts(roles),
            groups: Vec::new(),
            organizations: Vec::new(),
        }))
    }
}

fn role_facts(roles: Vec<roles::Model>) -> Vec<RoleFacts> {
    roles
        .into_iter()
        .map(|r| RoleFacts {
            id: r.id,
            name: r.name,
        })
        .collect()
}

impl PrincipalSliceRepository for PostgresPrincipalSliceRepository {
    async fn load(
        &self,
        realm: &RealmScope,
        principal: &EntityRef,
    ) -> Result<Option<PrincipalSlice>, CoreError> {
        let ty = principal.ty.as_str();
        if ty != USER_TYPE && ty != CLIENT_TYPE {
            return Err(CoreError::InvalidAuthorizationRequest(format!(
                "{ty} cannot be evaluated as a principal"
            )));
        }
        let Ok(id) = Uuid::parse_str(&principal.id) else {
            return Ok(None);
        };
        let realm_id = Uuid::from(realm.id());

        if ty == USER_TYPE {
            let user = users::Entity::find_by_id(id)
                .filter(users::Column::RealmId.eq(realm_id))
                .one(&self.db)
                .await
                .map_err(internal("load principal user"))?;
            match user {
                Some(user) => self.load_user(realm_id, user).await.map(Some),
                None => Ok(None),
            }
        } else {
            let client = clients::Entity::find_by_id(id)
                .filter(clients::Column::RealmId.eq(realm_id))
                .one(&self.db)
                .await
                .map_err(internal("load principal client"))?;
            match client {
                Some(client) => self.load_client(realm_id, client).await,
                None => Ok(None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ferriskey_authz::entities::EntityType;
    use ferriskey_domain::realm::{Realm, RealmId};
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;

    use super::*;

    struct Fixture {
        repository: PostgresPrincipalSliceRepository,
        realm_a: RealmScope,
        realm_b: RealmScope,
        pool: sqlx::PgPool,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("principal_slice_test_{}", Uuid::new_v4().simple());

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
            repository: PostgresPrincipalSliceRepository::new(db),
            realm_a,
            realm_b,
            pool,
        }
    }

    fn realm_uuid(realm: &RealmScope) -> Uuid {
        Uuid::from(realm.id())
    }

    async fn user(f: &Fixture, realm: &RealmScope, username: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, realm_id, username, email, email_verified) VALUES ($1, $2, $3, $4, TRUE)",
        )
        .bind(id)
        .bind(realm_uuid(realm))
        .bind(username)
        .bind(format!("{username}@example.test"))
        .execute(&f.pool)
        .await
        .expect("insert user");
        id
    }

    async fn role(f: &Fixture, realm: &RealmScope, name: &str, holder: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO roles (id, name, realm_id) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(name)
            .bind(realm_uuid(realm))
            .execute(&f.pool)
            .await
            .expect("insert role");
        sqlx::query("INSERT INTO user_role (user_id, role_id) VALUES ($1, $2)")
            .bind(holder)
            .bind(id)
            .execute(&f.pool)
            .await
            .expect("assign role");
        id
    }

    async fn organization(f: &Fixture, realm: &RealmScope, alias: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO organizations (id, realm_id, name, alias) VALUES ($1, $2, $3, $3)",
        )
        .bind(id)
        .bind(realm_uuid(realm))
        .bind(alias)
        .execute(&f.pool)
        .await
        .expect("insert organization");
        id
    }

    async fn group(f: &Fixture, org: Uuid, parent: Option<Uuid>, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO organization_groups (id, organization_id, parent_group_id, name) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(org)
        .bind(parent)
        .bind(name)
        .execute(&f.pool)
        .await
        .expect("insert group");
        id
    }

    async fn join_group(f: &Fixture, group: Uuid, user: Uuid) {
        sqlx::query("INSERT INTO organization_group_members (id, group_id, user_id) VALUES (gen_random_uuid(), $1, $2)")
            .bind(group)
            .bind(user)
            .execute(&f.pool)
            .await
            .expect("join group");
    }

    async fn join_org(f: &Fixture, org: Uuid, user: Uuid) {
        sqlx::query("INSERT INTO organization_members (id, organization_id, user_id) VALUES (gen_random_uuid(), $1, $2)")
            .bind(org)
            .bind(user)
            .execute(&f.pool)
            .await
            .expect("join organization");
    }

    fn user_ref(id: impl ToString) -> EntityRef {
        EntityRef::new(EntityType::custom(USER_TYPE), id.to_string())
    }

    fn sorted<T: Ord>(mut values: Vec<T>) -> Vec<T> {
        values.sort();
        values
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_users_facts_include_ancestor_groups_and_their_organizations() {
        let f = setup().await;
        let alice = user(&f, &f.realm_a, "alice").await;
        let admin = role(&f, &f.realm_a, "admin", alice).await;
        let stranger = user(&f, &f.realm_b, "stranger").await;
        role(&f, &f.realm_b, "foreign", alice).await;
        role(&f, &f.realm_b, "other", stranger).await;
        for (key, value) in [("team", "blue"), ("tier", "gold")] {
            sqlx::query(
                "INSERT INTO user_attributes (id, user_id, realm_id, key, value) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(Uuid::new_v4())
            .bind(alice)
            .bind(realm_uuid(&f.realm_a))
            .bind(key)
            .bind(value)
            .execute(&f.pool)
            .await
            .expect("insert attribute");
        }
        let acme = organization(&f, &f.realm_a, "acme").await;
        let globex = organization(&f, &f.realm_a, "globex").await;
        let initech = organization(&f, &f.realm_a, "initech").await;
        let root = group(&f, globex, None, "root").await;
        let middle = group(&f, globex, Some(root), "middle").await;
        let leaf = group(&f, globex, Some(middle), "leaf").await;
        let unrelated = group(&f, initech, None, "unrelated").await;
        join_group(&f, leaf, alice).await;
        join_group(&f, unrelated, stranger).await;
        join_org(&f, acme, alice).await;

        let slice = f
            .repository
            .load(&f.realm_a, &user_ref(alice))
            .await
            .expect("load")
            .expect("found");

        let PrincipalFacts::User(facts) = &slice.principal else {
            panic!("expected a user");
        };
        assert_eq!(facts.username, "alice");
        assert_eq!(facts.email.as_deref(), Some("alice@example.test"));
        assert!(facts.email_verified && facts.enabled);
        assert_eq!(
            facts.tags,
            BTreeMap::from([
                ("team".to_string(), "blue".to_string()),
                ("tier".to_string(), "gold".to_string())
            ])
        );
        assert_eq!(facts.role_ids, vec![admin]);
        assert_eq!(
            slice.roles,
            vec![RoleFacts {
                id: admin,
                name: "admin".to_string()
            }]
        );
        assert_eq!(facts.group_ids, vec![leaf]);
        assert_eq!(facts.organization_ids, vec![acme]);
        assert_eq!(
            sorted(slice.groups.iter().map(|g| g.id).collect()),
            sorted(vec![root, middle, leaf])
        );
        let middle_facts = slice
            .groups
            .iter()
            .find(|g| g.id == middle)
            .expect("middle");
        assert_eq!(middle_facts.parent_id, Some(root));
        assert_eq!(middle_facts.organization_id, globex);
        assert_eq!(
            sorted(slice.organizations.iter().map(|o| o.id).collect()),
            sorted(vec![acme, globex])
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_group_parent_cycle_terminates() {
        let f = setup().await;
        let alice = user(&f, &f.realm_a, "alice").await;
        let org = organization(&f, &f.realm_a, "acme").await;
        let first = group(&f, org, None, "first").await;
        let second = group(&f, org, Some(first), "second").await;
        sqlx::query("UPDATE organization_groups SET parent_group_id = $1 WHERE id = $2")
            .bind(second)
            .bind(first)
            .execute(&f.pool)
            .await
            .expect("close the cycle");
        join_group(&f, first, alice).await;

        let slice = f
            .repository
            .load(&f.realm_a, &user_ref(alice))
            .await
            .expect("load")
            .expect("found");

        assert_eq!(
            sorted(slice.groups.iter().map(|g| g.id).collect()),
            sorted(vec![first, second])
        );
        assert_eq!(slice.organizations.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_service_account_clients_roles_are_those_of_its_account() {
        let f = setup().await;
        let client = Uuid::new_v4();
        let plain = Uuid::new_v4();
        for (id, service_account) in [(client, true), (plain, false)] {
            sqlx::query(
                "INSERT INTO clients (id, realm_id, name, client_id, protocol, client_type, service_account_enabled) VALUES ($1, $2, $3, $3, 'openid-connect', 'confidential', $4)",
            )
            .bind(id)
            .bind(realm_uuid(&f.realm_a))
            .bind(format!("client-{id}"))
            .bind(service_account)
            .execute(&f.pool)
            .await
            .expect("insert client");
        }
        let account = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, realm_id, client_id, username) VALUES ($1, $2, $3, 'service-account-x')",
        )
        .bind(account)
        .bind(realm_uuid(&f.realm_a))
        .bind(client)
        .execute(&f.pool)
        .await
        .expect("insert service account");
        let reader = role(&f, &f.realm_a, "reader", account).await;
        let reference = |id: Uuid| EntityRef::new(EntityType::custom(CLIENT_TYPE), id.to_string());

        let slice = f
            .repository
            .load(&f.realm_a, &reference(client))
            .await
            .expect("load")
            .expect("found");
        let PrincipalFacts::Client(facts) = &slice.principal else {
            panic!("expected a client");
        };
        assert_eq!(facts.id, client);
        assert_eq!(facts.role_ids, vec![reader]);
        assert_eq!(slice.roles.len(), 1);

        assert!(
            f.repository
                .load(&f.realm_a, &reference(plain))
                .await
                .expect("load plain")
                .is_none()
        );
        assert!(
            f.repository
                .load(&f.realm_b, &reference(client))
                .await
                .expect("load from other realm")
                .is_none()
        );

        sqlx::query("UPDATE users SET enabled = false WHERE id = $1")
            .bind(account)
            .execute(&f.pool)
            .await
            .expect("disable service account");
        assert!(
            f.repository
                .load(&f.realm_a, &reference(client))
                .await
                .expect("load disabled")
                .is_none()
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn unknown_foreign_and_malformed_ids_are_not_found() {
        let f = setup().await;
        let alice = user(&f, &f.realm_a, "alice").await;

        for (realm, id) in [
            (&f.realm_b, alice.to_string()),
            (&f.realm_a, Uuid::new_v4().to_string()),
            (&f.realm_a, "not-a-uuid".to_string()),
        ] {
            assert!(
                f.repository
                    .load(realm, &user_ref(id))
                    .await
                    .expect("load")
                    .is_none()
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL"]
    async fn a_non_principal_type_is_an_invalid_request() {
        let f = setup().await;
        for ty in ["FerrisKey::Role", "App::Document"] {
            let subject = EntityRef::new(EntityType::custom(ty), Uuid::new_v4().to_string());
            let result = f.repository.load(&f.realm_a, &subject).await;
            assert!(
                matches!(result, Err(CoreError::InvalidAuthorizationRequest(_))),
                "got {result:?}"
            );
        }
    }
}

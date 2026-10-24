use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use tracing::error;
use uuid::Uuid;

use ferriskey_organization::{
    Group, GroupId, GroupTokenRepository, OrganizationId, TokenGroupMembership,
};

use crate::domain::common::entities::app_errors::CoreError;

macro_rules! token_membership_sql {
    ($user_id:literal) => {
        concat!(
            r#"
WITH RECURSIVE user_groups AS (
    SELECT g.id, g.organization_id, g.parent_group_id, g.name, g.description, g.created_at, g.updated_at, TRUE AS direct
    FROM organization_groups g
    JOIN organization_group_members m ON m.group_id = g.id
    WHERE m.user_id = "#,
            $user_id,
            r#"
  UNION
    SELECT p.id, p.organization_id, p.parent_group_id, p.name, p.description, p.created_at, p.updated_at, FALSE
    FROM organization_groups p
    JOIN user_groups ug ON ug.parent_group_id = p.id
),
groups AS (
    SELECT id, organization_id, parent_group_id, name, description, created_at, updated_at, bool_or(direct) AS direct
    FROM user_groups
    GROUP BY id, organization_id, parent_group_id, name, description, created_at, updated_at
),
role_ids AS (
    SELECT DISTINCT ug.organization_id, gr.role_id, 'group' AS source
    FROM organization_group_roles gr
    JOIN user_groups ug ON ug.id = gr.group_id
  UNION ALL
    SELECT m.organization_id, mr.role_id, 'member' AS source
    FROM organization_member_roles mr
    JOIN organization_members m ON m.id = mr.organization_member_id
    WHERE m.user_id = "#,
            $user_id,
            r#"
)
SELECT 'group' AS kind, id, organization_id, parent_group_id, name, description, created_at, updated_at, direct,
       NULL::uuid AS role_id, NULL::text AS source
FROM groups
UNION ALL
SELECT 'role' AS kind, NULL::uuid, organization_id, NULL::uuid, NULL::text, NULL::text, NULL::timestamptz, NULL::timestamptz, NULL::boolean,
       role_id, source
FROM role_ids
"#
        )
    };
}

/// The user's direct groups plus every ancestor, resolved with a recursive CTE; `direct` marks
/// the groups the user is a member of without ancestor expansion.
const TOKEN_MEMBERSHIP_SQL: &str = token_membership_sql!("$1");

const SERVICE_ACCOUNT_MEMBERSHIP_SQL: &str =
    token_membership_sql!("(SELECT id FROM users WHERE client_id = $1)");

const EFFECTIVE_ROLE_IDS_SQL: &str = r#"
WITH RECURSIVE user_groups AS (
    SELECT g.id, g.parent_group_id
    FROM organization_groups g
    JOIN organization_group_members m ON m.group_id = g.id
    WHERE m.user_id = $1
  UNION
    SELECT p.id, p.parent_group_id
    FROM organization_groups p
    JOIN user_groups ug ON ug.parent_group_id = p.id
)
SELECT DISTINCT gr.role_id
FROM organization_group_roles gr
JOIN user_groups ug ON ug.id = gr.group_id
"#;

#[derive(Debug, Clone)]
pub struct PostgresGroupTokenRepository {
    pub db: DatabaseConnection,
}

impl PostgresGroupTokenRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PostgresGroupTokenRepository {
    async fn load_membership(
        &self,
        sql: &'static str,
        key: Uuid,
    ) -> Result<TokenGroupMembership, CoreError> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                sql,
                [key.into()],
            ))
            .await
            .map_err(|e| {
                error!("Failed to resolve the token membership of a user: {}", e);
                CoreError::InternalServerError
            })?;

        let mut membership = TokenGroupMembership::default();
        for row in rows {
            let kind: String = row.try_get("", "kind").map_err(map_row_err)?;
            if kind == "role" {
                let org_id: Uuid = row.try_get("", "organization_id").map_err(map_row_err)?;
                let role_id: Uuid = row.try_get("", "role_id").map_err(map_row_err)?;
                let source: String = row.try_get("", "source").map_err(map_row_err)?;
                match source.as_str() {
                    "group" => membership.group_role_ids_by_org.push((org_id, role_id)),
                    _ => membership.member_role_ids_by_org.push((org_id, role_id)),
                }
                continue;
            }

            let id: Uuid = row.try_get("", "id").map_err(map_row_err)?;
            let direct: bool = row.try_get("", "direct").map_err(map_row_err)?;
            let created_at: DateTime<FixedOffset> =
                row.try_get("", "created_at").map_err(map_row_err)?;
            let updated_at: DateTime<FixedOffset> =
                row.try_get("", "updated_at").map_err(map_row_err)?;
            let parent: Option<Uuid> = row.try_get("", "parent_group_id").map_err(map_row_err)?;

            if direct {
                membership.direct_group_ids.insert(id);
            }
            membership.effective_groups.push(Group {
                id: GroupId::new(id),
                organization_id: OrganizationId::new(
                    row.try_get("", "organization_id").map_err(map_row_err)?,
                ),
                parent_group_id: parent.map(GroupId::new),
                name: row.try_get("", "name").map_err(map_row_err)?,
                description: row.try_get("", "description").map_err(map_row_err)?,
                created_at: created_at.with_timezone(&Utc),
                updated_at: updated_at.with_timezone(&Utc),
            });
        }

        Ok(membership)
    }
}

impl GroupTokenRepository for PostgresGroupTokenRepository {
    async fn token_membership(&self, user_id: Uuid) -> Result<TokenGroupMembership, CoreError> {
        self.load_membership(TOKEN_MEMBERSHIP_SQL, user_id).await
    }

    async fn token_membership_of_service_account(
        &self,
        client_id: Uuid,
    ) -> Result<TokenGroupMembership, CoreError> {
        self.load_membership(SERVICE_ACCOUNT_MEMBERSHIP_SQL, client_id)
            .await
    }

    async fn list_effective_role_ids_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, CoreError> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                EFFECTIVE_ROLE_IDS_SQL,
                [user_id.into()],
            ))
            .await
            .map_err(|e| {
                error!("Failed to resolve effective role ids for user: {}", e);
                CoreError::InternalServerError
            })?;

        rows.into_iter()
            .map(|row| row.try_get::<Uuid>("", "role_id").map_err(map_row_err))
            .collect()
    }
}

fn map_row_err(e: sea_orm::DbErr) -> CoreError {
    error!("Failed to read group row: {}", e);
    CoreError::InternalServerError
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;
    use uuid::Uuid;

    use ferriskey_organization::GroupTokenRepository;

    use super::PostgresGroupTokenRepository;

    struct Fixture {
        repository: PostgresGroupTokenRepository,
        pool: sqlx::PgPool,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });

        let schema = format!("group_token_repository_test_{}", Uuid::new_v4().simple());

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
            repository: PostgresGroupTokenRepository::new(db),
            pool,
        }
    }

    struct Seed {
        user_id: Uuid,
        organization_id: Uuid,
        parent_group_id: Uuid,
        child_group_id: Uuid,
        inherited_role_id: Uuid,
        member_role_id: Uuid,
    }

    async fn run(pool: &sqlx::PgPool, sql: &str, binds: &[Uuid]) {
        let mut query = sqlx::query(sql);
        for bind in binds {
            query = query.bind(*bind);
        }
        query
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("{sql}: {e}"));
    }

    async fn seed(pool: &sqlx::PgPool) -> Seed {
        let realm_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let parent_group_id = Uuid::new_v4();
        let child_group_id = Uuid::new_v4();
        let inherited_role_id = Uuid::new_v4();
        let member_role_id = Uuid::new_v4();
        let membership_id = Uuid::new_v4();

        run(pool, "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, 'org-realm', NOW(), NOW())", &[realm_id]).await;
        run(pool, "INSERT INTO users (id, realm_id, username, firstname, lastname, email) VALUES ($1, $2, 'alice', '', '', 'alice@example.com')", &[user_id, realm_id]).await;
        run(
            pool,
            "INSERT INTO organizations (id, realm_id, name, alias) VALUES ($1, $2, 'Acme', 'acme')",
            &[organization_id, realm_id],
        )
        .await;
        run(pool, "INSERT INTO organization_groups (id, organization_id, parent_group_id, name) VALUES ($1, $2, NULL, 'staff')", &[parent_group_id, organization_id]).await;
        run(pool, "INSERT INTO organization_groups (id, organization_id, parent_group_id, name) VALUES ($1, $2, $3, 'engineering')", &[child_group_id, organization_id, parent_group_id]).await;
        run(
            pool,
            "INSERT INTO organization_group_members (id, group_id, user_id) VALUES ($1, $2, $3)",
            &[Uuid::new_v4(), child_group_id, user_id],
        )
        .await;
        run(
            pool,
            "INSERT INTO roles (id, name, realm_id) VALUES ($1, 'inherited', $2)",
            &[inherited_role_id, realm_id],
        )
        .await;
        run(
            pool,
            "INSERT INTO roles (id, name, realm_id) VALUES ($1, 'member', $2)",
            &[member_role_id, realm_id],
        )
        .await;
        run(
            pool,
            "INSERT INTO organization_group_roles (id, group_id, role_id) VALUES ($1, $2, $3)",
            &[Uuid::new_v4(), parent_group_id, inherited_role_id],
        )
        .await;
        run(
            pool,
            "INSERT INTO organization_members (id, organization_id, user_id) VALUES ($1, $2, $3)",
            &[membership_id, organization_id, user_id],
        )
        .await;
        run(pool, "INSERT INTO organization_member_roles (id, organization_member_id, role_id) VALUES ($1, $2, $3)", &[Uuid::new_v4(), membership_id, member_role_id]).await;

        Seed {
            user_id,
            organization_id,
            parent_group_id,
            child_group_id,
            inherited_role_id,
            member_role_id,
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn token_membership_resolves_ancestors_direct_groups_and_both_role_sources() {
        let fixture = setup().await;
        let seed = seed(&fixture.pool).await;

        let membership = fixture
            .repository
            .token_membership(seed.user_id)
            .await
            .expect("token membership");

        let group_ids: HashSet<Uuid> = membership
            .effective_groups
            .iter()
            .map(|group| group.id.as_uuid())
            .collect();
        assert_eq!(
            group_ids,
            HashSet::from([seed.parent_group_id, seed.child_group_id])
        );
        assert_eq!(
            membership.direct_group_ids,
            HashSet::from([seed.child_group_id])
        );
        assert_eq!(
            membership.group_role_ids_by_org,
            vec![(seed.organization_id, seed.inherited_role_id)]
        );
        assert_eq!(
            membership.member_role_ids_by_org,
            vec![(seed.organization_id, seed.member_role_id)]
        );
        assert_eq!(
            membership.effective_role_ids(),
            vec![seed.inherited_role_id]
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_user_without_groups_or_memberships_gets_an_empty_membership() {
        let fixture = setup().await;
        let seed = seed(&fixture.pool).await;
        let loner = Uuid::new_v4();
        run(
            &fixture.pool,
            "INSERT INTO users (id, realm_id, username, firstname, lastname, email) \
             SELECT $1, realm_id, 'bob', '', '', 'bob@example.com' FROM users WHERE id = $2",
            &[loner, seed.user_id],
        )
        .await;

        let membership = fixture
            .repository
            .token_membership(loner)
            .await
            .expect("token membership");

        assert_eq!(membership, Default::default());
    }
}

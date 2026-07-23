use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use tracing::error;
use uuid::Uuid;

use ferriskey_organization::{Group, GroupId, GroupTokenRepository, OrganizationId};

use crate::domain::common::entities::app_errors::CoreError;

/// The user's direct groups plus every ancestor, resolved with a recursive CTE.
const EFFECTIVE_GROUPS_SQL: &str = r#"
WITH RECURSIVE user_groups AS (
    SELECT g.id, g.organization_id, g.parent_group_id, g.name, g.description, g.created_at, g.updated_at
    FROM organization_groups g
    JOIN organization_group_members m ON m.group_id = g.id
    WHERE m.user_id = $1
  UNION
    SELECT p.id, p.organization_id, p.parent_group_id, p.name, p.description, p.created_at, p.updated_at
    FROM organization_groups p
    JOIN user_groups ug ON ug.parent_group_id = p.id
)
SELECT DISTINCT id, organization_id, parent_group_id, name, description, created_at, updated_at
FROM user_groups
"#;

/// Ids of the groups the user is a direct member of (no ancestor expansion).
const DIRECT_GROUP_IDS_SQL: &str = r#"
SELECT group_id
FROM organization_group_members
WHERE user_id = $1
"#;

/// Distinct role ids inherited from the user's effective (recursive) groups.
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

/// `(organization_id, role_id)` pairs inherited from the user's effective (recursive) groups,
/// tagged with the organization each group belongs to. Powers the org-scoped role claim.
const EFFECTIVE_GROUP_ROLE_IDS_BY_ORG_SQL: &str = r#"
WITH RECURSIVE user_groups AS (
    SELECT g.id, g.organization_id, g.parent_group_id
    FROM organization_groups g
    JOIN organization_group_members m ON m.group_id = g.id
    WHERE m.user_id = $1
  UNION
    SELECT p.id, p.organization_id, p.parent_group_id
    FROM organization_groups p
    JOIN user_groups ug ON ug.parent_group_id = p.id
)
SELECT DISTINCT ug.organization_id, gr.role_id
FROM organization_group_roles gr
JOIN user_groups ug ON ug.id = gr.group_id
"#;

/// `(organization_id, role_id)` pairs assigned directly to the user's organization memberships.
const MEMBER_ROLE_IDS_BY_ORG_SQL: &str = r#"
SELECT m.organization_id, mr.role_id
FROM organization_member_roles mr
JOIN organization_members m ON m.id = mr.organization_member_id
WHERE m.user_id = $1
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

impl GroupTokenRepository for PostgresGroupTokenRepository {
    async fn list_effective_groups_for_user(&self, user_id: Uuid) -> Result<Vec<Group>, CoreError> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                EFFECTIVE_GROUPS_SQL,
                [user_id.into()],
            ))
            .await
            .map_err(|e| {
                error!("Failed to resolve effective groups for user: {}", e);
                CoreError::InternalServerError
            })?;

        let mut groups = Vec::with_capacity(rows.len());
        for row in rows {
            let created_at: DateTime<FixedOffset> =
                row.try_get("", "created_at").map_err(map_row_err)?;
            let updated_at: DateTime<FixedOffset> =
                row.try_get("", "updated_at").map_err(map_row_err)?;
            let parent: Option<Uuid> = row.try_get("", "parent_group_id").map_err(map_row_err)?;

            groups.push(Group {
                id: GroupId::new(row.try_get("", "id").map_err(map_row_err)?),
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

        Ok(groups)
    }

    async fn list_direct_group_ids_for_user(&self, user_id: Uuid) -> Result<Vec<Uuid>, CoreError> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                DIRECT_GROUP_IDS_SQL,
                [user_id.into()],
            ))
            .await
            .map_err(|e| {
                error!("Failed to resolve direct groups for user: {}", e);
                CoreError::InternalServerError
            })?;

        rows.into_iter()
            .map(|row| row.try_get::<Uuid>("", "group_id").map_err(map_row_err))
            .collect()
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

    async fn list_effective_group_role_ids_by_org_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<(Uuid, Uuid)>, CoreError> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                EFFECTIVE_GROUP_ROLE_IDS_BY_ORG_SQL,
                [user_id.into()],
            ))
            .await
            .map_err(|e| {
                error!(
                    "Failed to resolve effective group roles by org for user: {}",
                    e
                );
                CoreError::InternalServerError
            })?;

        rows.into_iter()
            .map(|row| {
                let org_id: Uuid = row.try_get("", "organization_id").map_err(map_row_err)?;
                let role_id: Uuid = row.try_get("", "role_id").map_err(map_row_err)?;
                Ok((org_id, role_id))
            })
            .collect()
    }

    async fn list_member_role_ids_by_org_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<(Uuid, Uuid)>, CoreError> {
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                MEMBER_ROLE_IDS_BY_ORG_SQL,
                [user_id.into()],
            ))
            .await
            .map_err(|e| {
                error!("Failed to resolve member roles by org for user: {}", e);
                CoreError::InternalServerError
            })?;

        rows.into_iter()
            .map(|row| {
                let org_id: Uuid = row.try_get("", "organization_id").map_err(map_row_err)?;
                let role_id: Uuid = row.try_get("", "role_id").map_err(map_row_err)?;
                Ok((org_id, role_id))
            })
            .collect()
    }
}

fn map_row_err(e: sea_orm::DbErr) -> CoreError {
    error!("Failed to read group row: {}", e);
    CoreError::InternalServerError
}

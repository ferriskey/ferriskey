use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Statement,
};
use uuid::Uuid;

use ferriskey_compass::{
    entities::{CompassFlow, FlowStatus},
    ports::CompassFlowRepository,
    value_objects::{DailyActivityStats, DailyActivityStatsFilter, FlowFilter, FlowStats},
};
use ferriskey_domain::realm::RealmId;

use crate::domain::common::entities::app_errors::CoreError;
use crate::entity::{compass_flow_steps, compass_flows};

#[derive(Debug, Clone)]
pub struct PostgresCompassFlowRepository {
    pub db: DatabaseConnection,
}

impl PostgresCompassFlowRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn apply_filter(
        query: sea_orm::Select<compass_flows::Entity>,
        realm_id: RealmId,
        filter: &FlowFilter,
    ) -> sea_orm::Select<compass_flows::Entity> {
        let mut query = query.filter(compass_flows::Column::RealmId.eq::<Uuid>(realm_id.into()));

        if let Some(ref client_id) = filter.client_id {
            query = query.filter(compass_flows::Column::ClientId.eq(client_id.clone()));
        }

        if let Some(user_id) = filter.user_id {
            query = query.filter(compass_flows::Column::UserId.eq(user_id));
        }

        if let Some(ref grant_type) = filter.grant_type {
            query = query.filter(compass_flows::Column::GrantType.eq(grant_type.clone()));
        }

        if let Some(ref status) = filter.status {
            query = query.filter(compass_flows::Column::Status.eq(status.clone()));
        }

        if let Some(from) = filter.from_timestamp {
            query = query.filter(compass_flows::Column::StartedAt.gte(from.naive_utc()));
        }

        if let Some(to) = filter.to_timestamp {
            query = query.filter(compass_flows::Column::StartedAt.lte(to.naive_utc()));
        }

        query
    }
}

impl CompassFlowRepository for PostgresCompassFlowRepository {
    async fn create_flow(&self, flow: CompassFlow) -> Result<(), CoreError> {
        let active_model: compass_flows::ActiveModel = flow.into();

        compass_flows::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create compass flow: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn update_flow_status(
        &self,
        flow_id: Uuid,
        status: FlowStatus,
        completed_at: DateTime<Utc>,
        duration_ms: Option<i64>,
        user_id: Option<Uuid>,
    ) -> Result<(), CoreError> {
        let model = compass_flows::Entity::find_by_id(flow_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find compass flow: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active_model: compass_flows::ActiveModel = model.into();
        active_model.status = Set(status.to_string());
        active_model.completed_at = Set(Some(completed_at.naive_utc()));
        active_model.duration_ms = Set(duration_ms);
        active_model.user_id = Set(user_id);

        compass_flows::Entity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update compass flow status: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn get_flows(
        &self,
        realm_id: RealmId,
        filter: FlowFilter,
    ) -> Result<Vec<CompassFlow>, CoreError> {
        let query = Self::apply_filter(compass_flows::Entity::find(), realm_id, &filter)
            .order_by_desc(compass_flows::Column::StartedAt);

        let query = if let Some(limit) = filter.limit {
            query.limit(limit as u64)
        } else {
            query
        };

        let query = if let Some(offset) = filter.offset {
            query.offset(offset as u64)
        } else {
            query
        };

        let results = query
            .find_with_related(compass_flow_steps::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get compass flows: {}", e);
                CoreError::InternalServerError
            })?;

        let flows = results
            .into_iter()
            .map(|(flow_model, step_models)| {
                let mut flow: CompassFlow = flow_model.into();
                flow.steps = step_models.into_iter().map(|s| s.into()).collect();
                flow
            })
            .collect();

        Ok(flows)
    }

    async fn get_flow_by_id(&self, flow_id: Uuid) -> Result<Option<CompassFlow>, CoreError> {
        let result = compass_flows::Entity::find_by_id(flow_id)
            .find_with_related(compass_flow_steps::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get compass flow by id: {}", e);
                CoreError::InternalServerError
            })?;

        let flow = result.into_iter().next().map(|(flow_model, step_models)| {
            let mut flow: CompassFlow = flow_model.into();
            flow.steps = step_models.into_iter().map(|s| s.into()).collect();
            flow
        });

        Ok(flow)
    }

    async fn count_flows(&self, realm_id: RealmId, filter: FlowFilter) -> Result<i64, CoreError> {
        let query = Self::apply_filter(compass_flows::Entity::find(), realm_id, &filter);

        let count = query.count(&self.db).await.map_err(|e| {
            tracing::error!("Failed to count compass flows: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(count as i64)
    }

    async fn purge_old_flows(&self, older_than: DateTime<Utc>) -> Result<u64, CoreError> {
        let result = compass_flows::Entity::delete_many()
            .filter(compass_flows::Column::StartedAt.lt(older_than.naive_utc()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to purge old compass flows: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected)
    }

    async fn get_stats(&self, realm_id: RealmId) -> Result<FlowStats, CoreError> {
        let realm_uuid: Uuid = realm_id.into();

        let total = compass_flows::Entity::find()
            .filter(compass_flows::Column::RealmId.eq(realm_uuid))
            .count(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count total flows: {}", e);
                CoreError::InternalServerError
            })? as i64;

        let success_count = compass_flows::Entity::find()
            .filter(compass_flows::Column::RealmId.eq(realm_uuid))
            .filter(compass_flows::Column::Status.eq("success"))
            .count(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count success flows: {}", e);
                CoreError::InternalServerError
            })? as i64;

        let failure_count = compass_flows::Entity::find()
            .filter(compass_flows::Column::RealmId.eq(realm_uuid))
            .filter(compass_flows::Column::Status.eq("failure"))
            .count(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count failure flows: {}", e);
                CoreError::InternalServerError
            })? as i64;

        let pending_count = compass_flows::Entity::find()
            .filter(compass_flows::Column::RealmId.eq(realm_uuid))
            .filter(compass_flows::Column::Status.eq("pending"))
            .count(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count pending flows: {}", e);
                CoreError::InternalServerError
            })? as i64;

        let avg_duration_ms = self
            .db
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "SELECT AVG(duration_ms)::float8 as avg_duration FROM compass_flows WHERE realm_id = $1 AND duration_ms IS NOT NULL",
                [realm_uuid.into()],
            ))
            .await
            .map_err(|e| {
                tracing::error!("Failed to get avg duration: {}", e);
                CoreError::InternalServerError
            })?
            .and_then(|row| {
                use sea_orm::TryGetable;
                Option::<f64>::try_get_by(&row, "avg_duration").ok().flatten()
            });

        Ok(FlowStats {
            total,
            success_count,
            failure_count,
            pending_count,
            avg_duration_ms,
        })
    }

    async fn get_daily_activity_stats(
        &self,
        realm_id: RealmId,
        filter: DailyActivityStatsFilter,
    ) -> Result<Vec<DailyActivityStats>, CoreError> {
        let realm_uuid: Uuid = realm_id.into();

        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"
                WITH days AS (
                    SELECT generate_series($2::date, $3::date, interval '1 day')::date AS day
                ),
                flow_stats AS (
                    SELECT
                        started_at::date AS day,
                        COUNT(*)::bigint AS total_flows,
                        COUNT(*) FILTER (WHERE status = 'success')::bigint AS logins,
                        COUNT(*) FILTER (WHERE status = 'failure')::bigint AS login_failures,
                        COUNT(*) FILTER (WHERE status = 'pending')::bigint AS pending_logins,
                        COUNT(*) FILTER (WHERE status = 'expired')::bigint AS expired_logins,
                        COUNT(DISTINCT user_id) FILTER (
                            WHERE status = 'success' AND user_id IS NOT NULL
                        )::bigint AS unique_login_users,
                        CAST(
                            AVG(duration_ms) FILTER (
                                WHERE status = 'success' AND duration_ms IS NOT NULL
                            ) AS float8
                        ) AS avg_login_duration_ms
                    FROM compass_flows
                    WHERE realm_id = $1
                        AND started_at >= $2::date
                        AND started_at < ($3::date + interval '1 day')
                        AND ($4::text IS NULL OR client_id = $4)
                        AND ($5::uuid IS NULL OR user_id = $5)
                        AND ($6::text IS NULL OR grant_type = $6)
                    GROUP BY started_at::date
                ),
                signup_stats AS (
                    SELECT
                        created_at::date AS day,
                        COUNT(*)::bigint AS signups
                    FROM users
                    WHERE realm_id = $1
                        AND created_at >= $2::date
                        AND created_at < ($3::date + interval '1 day')
                    GROUP BY created_at::date
                )
                SELECT
                    to_char(days.day, 'YYYY-MM-DD') AS date,
                    COALESCE(signup_stats.signups, 0)::bigint AS signups,
                    COALESCE(flow_stats.logins, 0)::bigint AS logins,
                    COALESCE(flow_stats.login_failures, 0)::bigint AS login_failures,
                    COALESCE(flow_stats.pending_logins, 0)::bigint AS pending_logins,
                    COALESCE(flow_stats.expired_logins, 0)::bigint AS expired_logins,
                    COALESCE(flow_stats.total_flows, 0)::bigint AS total_flows,
                    COALESCE(flow_stats.unique_login_users, 0)::bigint AS unique_login_users,
                    flow_stats.avg_login_duration_ms
                FROM days
                LEFT JOIN flow_stats ON flow_stats.day = days.day
                LEFT JOIN signup_stats ON signup_stats.day = days.day
                ORDER BY days.day ASC
                "#,
                vec![
                    realm_uuid.into(),
                    filter.from_date.to_string().into(),
                    filter.to_date.to_string().into(),
                    filter.client_id.into(),
                    filter.user_id.into(),
                    filter.grant_type.into(),
                ],
            ))
            .await
            .map_err(|e| {
                tracing::error!("Failed to get daily activity stats: {:?}", e);
                CoreError::InternalServerError
            })?;

        rows.into_iter()
            .map(|row| {
                use sea_orm::TryGetable;

                Ok(DailyActivityStats {
                    date: String::try_get_by(&row, "date").map_err(|e| {
                        tracing::error!("Failed to read daily activity date: {:?}", e);
                        CoreError::InternalServerError
                    })?,
                    signups: i64::try_get_by(&row, "signups").map_err(|e| {
                        tracing::error!("Failed to read daily signup count: {:?}", e);
                        CoreError::InternalServerError
                    })?,
                    logins: i64::try_get_by(&row, "logins").map_err(|e| {
                        tracing::error!("Failed to read daily login count: {:?}", e);
                        CoreError::InternalServerError
                    })?,
                    login_failures: i64::try_get_by(&row, "login_failures").map_err(|e| {
                        tracing::error!("Failed to read daily login failure count: {:?}", e);
                        CoreError::InternalServerError
                    })?,
                    pending_logins: i64::try_get_by(&row, "pending_logins").map_err(|e| {
                        tracing::error!("Failed to read daily pending login count: {:?}", e);
                        CoreError::InternalServerError
                    })?,
                    expired_logins: i64::try_get_by(&row, "expired_logins").map_err(|e| {
                        tracing::error!("Failed to read daily expired login count: {:?}", e);
                        CoreError::InternalServerError
                    })?,
                    total_flows: i64::try_get_by(&row, "total_flows").map_err(|e| {
                        tracing::error!("Failed to read daily flow count: {:?}", e);
                        CoreError::InternalServerError
                    })?,
                    unique_login_users: i64::try_get_by(&row, "unique_login_users").map_err(
                        |e| {
                            tracing::error!(
                                "Failed to read daily unique login user count: {:?}",
                                e
                            );
                            CoreError::InternalServerError
                        },
                    )?,
                    avg_login_duration_ms: Option::<f64>::try_get_by(&row, "avg_login_duration_ms")
                        .map_err(|e| {
                            tracing::error!("Failed to read daily avg login duration: {:?}", e);
                            CoreError::InternalServerError
                        })?,
                })
            })
            .collect()
    }
}

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QueryTrait, Select, Statement,
};
use uuid::Uuid;

use ferriskey_compass::{
    entities::{CompassFlow, CompassFlowStep, FlowStatus},
    ports::CompassFlowRepository,
    value_objects::{
        DailyActivityStats, DailyActivityStatsFilter, FlowFilter, FlowSortField, FlowStats,
    },
};
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::{RealmScope, Unscoped};

use crate::domain::common::entities::app_errors::CoreError;
use crate::entity::{compass_flow_steps, compass_flows};
use crate::infrastructure::pagination::{SortColumn, contains, paginate};

#[derive(Debug, Clone)]
pub struct PostgresCompassFlowRepository {
    pub db: DatabaseConnection,
}

impl PostgresCompassFlowRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl SortColumn<compass_flows::Entity> for FlowSortField {
    fn column(&self) -> compass_flows::Column {
        match self {
            FlowSortField::Status => compass_flows::Column::Status,
            FlowSortField::StartedAt => compass_flows::Column::StartedAt,
            FlowSortField::DurationMs => compass_flows::Column::DurationMs,
            FlowSortField::CreatedAt => compass_flows::Column::CreatedAt,
        }
    }
}

fn listing_select(realm_id: Uuid, filter: &FlowFilter) -> Select<compass_flows::Entity> {
    use compass_flows::Column;

    compass_flows::Entity::find()
        .filter(Column::RealmId.eq(realm_id))
        .apply_if(filter.client_id.as_deref(), |select, value| {
            select.filter(Column::ClientId.eq(value))
        })
        .apply_if(filter.user_id, |select, value| {
            select.filter(Column::UserId.eq(value))
        })
        .apply_if(filter.grant_type.as_deref(), |select, value| {
            select.filter(Column::GrantType.eq(value))
        })
        .apply_if(filter.status.as_ref(), |select, value| {
            select.filter(Column::Status.eq(value.to_string()))
        })
        .apply_if(filter.search.as_deref(), |select, value| {
            select.filter(contains(Column::IpAddress, value))
        })
        .apply_if(filter.ip_address.as_deref(), |select, value| {
            select.filter(contains(Column::IpAddress, value))
        })
        .apply_if(filter.identified, |select, value| {
            select.filter(if value {
                Column::UserId.is_not_null()
            } else {
                Column::UserId.is_null()
            })
        })
        .apply_if(filter.completed, |select, value| {
            select.filter(if value {
                Column::CompletedAt.is_not_null()
            } else {
                Column::CompletedAt.is_null()
            })
        })
        .apply_if(filter.from_timestamp, |select, value| {
            select.filter(Column::StartedAt.gte(value.naive_utc()))
        })
        .apply_if(filter.to_timestamp, |select, value| {
            select.filter(Column::StartedAt.lte(value.naive_utc()))
        })
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

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<FlowFilter, FlowSortField>,
    ) -> Result<Page<CompassFlow>, CoreError> {
        let select = listing_select(scope.id().into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            tracing::error!("Failed to list compass flows: {}", e);
            CoreError::InternalServerError
        })?;

        if models.is_empty() {
            return Ok(Page::new(Vec::new(), total, request.page, request.limit));
        }

        let flow_ids: Vec<Uuid> = models.iter().map(|model| model.id).collect();
        let mut steps = compass_flow_steps::Entity::find()
            .filter(compass_flow_steps::Column::FlowId.is_in(flow_ids))
            .order_by_asc(compass_flow_steps::Column::StartedAt)
            .order_by_asc(compass_flow_steps::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to load compass flow steps: {}", e);
                CoreError::InternalServerError
            })?
            .into_iter()
            .fold(
                HashMap::<Uuid, Vec<CompassFlowStep>>::new(),
                |mut acc, step| {
                    acc.entry(step.flow_id).or_default().push(step.into());
                    acc
                },
            );

        let flows = models
            .into_iter()
            .map(|model| {
                let flow_steps = steps.remove(&model.id).unwrap_or_default();
                let mut flow: CompassFlow = model.into();
                flow.steps = flow_steps;
                flow
            })
            .collect();

        Ok(Page::new(flows, total, request.page, request.limit))
    }

    async fn get_flow_by_id(
        &self,
        flow_id: Uuid,
    ) -> Result<Option<Unscoped<CompassFlow>>, CoreError> {
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
            Unscoped::new(flow)
        });

        Ok(flow)
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

    async fn expire_stale_flows(&self, started_before: DateTime<Utc>) -> Result<u64, CoreError> {
        let result = compass_flows::Entity::update_many()
            .col_expr(
                compass_flows::Column::Status,
                sea_orm::sea_query::Expr::value(FlowStatus::Expired.to_string()),
            )
            .filter(compass_flows::Column::Status.eq(FlowStatus::Pending.to_string()))
            .filter(compass_flows::Column::StartedAt.lt(started_before.naive_utc()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to expire stale compass flows: {}", e);
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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use ferriskey_compass::{entities::FlowStatus, value_objects::FlowFilter};
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;

    fn sql(filter: &FlowFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&FlowFilter::default());
        assert!(
            sql.ends_with(
                r#"WHERE "compass_flows"."realm_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn search_is_an_escaped_contains_match_on_the_ip_address() {
        let sql = sql(&FlowFilter {
            search: Some("a%_\\".to_string()),
            ..FlowFilter::default()
        });
        assert!(
            sql.contains(r#""compass_flows"."ip_address" ILIKE E'%a\\%\\_\\\\%'"#),
            "{sql}"
        );
        assert!(!sql.contains(r#""user_agent" ILIKE"#), "{sql}");
    }

    #[test]
    fn ip_address_is_an_escaped_contains_match() {
        let sql = sql(&FlowFilter {
            ip_address: Some("10.%_".to_string()),
            ..FlowFilter::default()
        });
        assert!(
            sql.contains(r#""compass_flows"."ip_address" ILIKE E'%10.\\%\\_%'"#),
            "{sql}"
        );
    }

    #[test]
    fn identifiers_grant_type_and_status_are_exact_matches() {
        let user_id = Uuid::max();
        let sql = sql(&FlowFilter {
            client_id: Some("web-app".to_string()),
            user_id: Some(user_id),
            grant_type: Some("password".to_string()),
            status: Some(FlowStatus::Expired),
            ..FlowFilter::default()
        });
        assert!(
            sql.contains(r#""compass_flows"."client_id" = 'web-app'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""compass_flows"."user_id" = 'ffffffff-ffff-ffff-ffff-ffffffffffff'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""compass_flows"."grant_type" = 'password'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""compass_flows"."status" = 'expired'"#),
            "{sql}"
        );
        assert!(!sql.contains("ILIKE"), "{sql}");
    }

    #[test]
    fn identified_and_completed_test_for_presence() {
        let present = sql(&FlowFilter {
            identified: Some(true),
            completed: Some(true),
            ..FlowFilter::default()
        });
        assert!(
            present.contains(r#""compass_flows"."user_id" IS NOT NULL"#),
            "{present}"
        );
        assert!(
            present.contains(r#""compass_flows"."completed_at" IS NOT NULL"#),
            "{present}"
        );

        let absent = sql(&FlowFilter {
            identified: Some(false),
            completed: Some(false),
            ..FlowFilter::default()
        });
        assert!(
            absent.contains(r#""compass_flows"."user_id" IS NULL"#),
            "{absent}"
        );
        assert!(
            absent.contains(r#""compass_flows"."completed_at" IS NULL"#),
            "{absent}"
        );
    }

    #[test]
    fn the_time_range_is_inclusive_on_started_at() {
        let sql = sql(&FlowFilter {
            from_timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 0, 5, 0).single(),
            to_timestamp: Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).single(),
            ..FlowFilter::default()
        });
        assert!(
            sql.contains(r#""compass_flows"."started_at" >= '2026-01-01 00:05:00.000000'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""compass_flows"."started_at" <= '2026-01-02 00:00:00.000000'"#),
            "{sql}"
        );
    }
}

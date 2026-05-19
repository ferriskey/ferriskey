use axum::{
    Extension,
    extract::{Path, Query, State},
};
use chrono::{Duration, NaiveDate, Utc};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    compass::{
        ports::CompassService,
        value_objects::{DailyActivityStats, DailyActivityStatsFilter},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

const MAX_DAILY_ACTIVITY_RANGE_DAYS: i64 = 366;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetDailyActivityStatsResponse {
    data: Vec<DailyActivityStats>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetDailyActivityStatsQuery {
    /// Start date, inclusive, in YYYY-MM-DD format. Defaults to the first day of a 30-day window ending at `to`.
    pub from: Option<String>,
    /// End date, inclusive, in YYYY-MM-DD format. Defaults to the current UTC day.
    pub to: Option<String>,
    pub client_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub grant_type: Option<String>,
}

fn parse_date(value: &str, field: &str) -> Result<NaiveDate, ApiError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest(format!("{field} must use the YYYY-MM-DD format").into()))
}

#[utoipa::path(
    get,
    summary = "Get Daily Activity Stats",
    path = "/compass/v1/activity/daily",
    tag = "compass",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        GetDailyActivityStatsQuery,
    ),
    responses(
        (status = 200, description = "Daily activity stats retrieved successfully", body = GetDailyActivityStatsResponse),
        (status = 400, description = "Invalid date range", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_daily_activity_stats(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<GetDailyActivityStatsQuery>,
) -> Result<Response<GetDailyActivityStatsResponse>, ApiError> {
    let default_to = Utc::now().date_naive();
    let to_date = match query.to.as_deref() {
        Some(to) => parse_date(to, "to")?,
        None => default_to,
    };
    let from_date = match query.from.as_deref() {
        Some(from) => parse_date(from, "from")?,
        None => to_date - Duration::days(29),
    };

    if from_date > to_date {
        return Err(ApiError::BadRequest(
            "from must be earlier than or equal to to".into(),
        ));
    }

    let range_days = (to_date - from_date).num_days() + 1;
    if range_days > MAX_DAILY_ACTIVITY_RANGE_DAYS {
        return Err(ApiError::BadRequest(
            format!("date range cannot exceed {MAX_DAILY_ACTIVITY_RANGE_DAYS} days").into(),
        ));
    }

    let stats = state
        .service
        .get_daily_activity_stats(
            identity,
            realm_name,
            DailyActivityStatsFilter {
                from_date,
                to_date,
                client_id: query.client_id,
                user_id: query.user_id,
                grant_type: query.grant_type,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetDailyActivityStatsResponse { data: stats }))
}

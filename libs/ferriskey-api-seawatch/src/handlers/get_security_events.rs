use axum::{
    Extension,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    seawatch::{
        SecurityEvent, SecurityEventFilter, SecurityEventType, ports::SecurityEventService,
        value_objects::FetchEventsInput,
    },
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;

/// A page of security events is capped at this many rows regardless of what
/// the caller asks for, so a wide-open query can't be used to pull an entire
/// realm's audit trail in one request.
const MAX_LIMIT: u32 = 1000;
const DEFAULT_LIMIT: u32 = 100;

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct GetSecurityEventsQuery {
    pub actor_id: Option<Uuid>,
    pub client_id: Option<Uuid>,
    /// Comma-separated `SecurityEventType` wire values, e.g. `login_failure,session_revoked`.
    pub event_types: Option<String>,
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl From<GetSecurityEventsQuery> for SecurityEventFilter {
    fn from(query: GetSecurityEventsQuery) -> Self {
        Self {
            user_id: None,
            client_id: query.client_id,
            actor_id: query.actor_id,
            event_types: query
                .event_types
                .map(|raw| raw.split(',').map(SecurityEventType::parse).collect()),
            from_timestamp: query.from_timestamp,
            to_timestamp: query.to_timestamp,
            ip_address: query.ip_address,
            limit: Some(query.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT)),
            offset: Some(query.offset.unwrap_or(0)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct GetSecurityEventsResponse {
    data: Vec<SecurityEvent>,
}

#[utoipa::path(
    get,
    summary = "Get Security Events",
    path = "/seawatch/v1/security-events",
    tag = "seawatch",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        GetSecurityEventsQuery,
    ),
    responses(
        (status = 200, description = "Security events retrieved successfully", body = GetSecurityEventsResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn get_security_events(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<GetSecurityEventsQuery>,
) -> Result<Response<GetSecurityEventsResponse>, ApiError> {
    let security_events = state
        .service
        .fetch_events(
            identity,
            FetchEventsInput {
                realm_name,
                filter: query.into(),
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetSecurityEventsResponse {
        data: security_events,
    }))
}

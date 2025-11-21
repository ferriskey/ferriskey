use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    seawatch::{SecurityEvent, ports::SecurityEventService, value_objects::FetchEventsInput},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

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
    ),
    responses(
        (status = 200, body = GetSecurityEventsResponse)
    )
)]
pub async fn get_security_events(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetSecurityEventsResponse>, ApiError> {
    let security_events = state
        .service
        .fetch_events(identity, FetchEventsInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetSecurityEventsResponse {
        data: security_events,
    }))
}

use axum::extract::{Path, State};
use ferriskey_core::domain::abyss::federation::ports::FederationService;
use uuid::Uuid;

use crate::application::http::{
    abyss::federation::dto::TestConnectionResponse,
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};

#[utoipa::path(
    post,
    path = "/federation/providers/{id}/test-connection",
    summary = "Test Federation Provider Connection",
    description = "Tests the connection to the external federation provider (LDAP, Kerberos, etc.) to verify configuration and connectivity",
    responses(
        (status = 200, description = "Connection test completed", body = TestConnectionResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Provider not found"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("id" = String, Path, description = "Provider ID")
    ),
    tag = "Federation"
)]
pub async fn test_connection(
    Path((_, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
) -> Result<Response<TestConnectionResponse>, ApiError> {
    let result = state
        .service
        .test_federation_connection(id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(result.into()))
}

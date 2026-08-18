use axum::Extension;
use axum::extract::{Path, State};
use ferriskey_core::domain::abyss::federation::ports::FederationService;
use uuid::Uuid;

use crate::federation::dto::TestConnectionResponse;
use ferriskey_api_core::api_entities::{api_error::ApiError, response::Response};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;

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
    tag = "federation"
)]
pub async fn test_connection(
    Path((realm_name, id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<TestConnectionResponse>, ApiError> {
    let result = state
        .service
        .test_federation_connection(identity, realm_name, id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(result.into()))
}

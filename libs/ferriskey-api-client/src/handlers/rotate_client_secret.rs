use axum::{
    Extension, Json,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::RotateClientSecretInput;
use ferriskey_core::domain::client::ports::ClientService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct RotateClientSecretRequest {
    /// Seconds the secret being replaced keeps working. Omitted or `0` cuts it
    /// off immediately. At most 30 days.
    #[serde(default)]
    #[schema(minimum = 0, maximum = 2_592_000)]
    pub grace_period_secs: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RotateClientSecretResponse {
    pub client_secret: String,
    /// When the replaced secret stops working. Absent when it was cut off.
    pub previous_secret_expires_at: Option<DateTime<Utc>>,
}

#[utoipa::path(
    post,
    path = "/{client_id}/client-secret/rotate",
    tag = "client",
    summary = "Rotate a confidential client's secret",
    description = "Generates a new secret for a confidential client. The secret it had keeps working for `grace_period_secs` (none by default, 30 days at most) so every deployment can switch over, then stops. Rotating again during that window replaces the previous secret, the older one stops at once. Requires ManageClients (or ManageRealm) and records a `client_secret_rotated` security event.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    request_body = RotateClientSecretRequest,
    responses(
        (status = 200, body = RotateClientSecretResponse),
        (status = 400, description = "Public client, or grace period out of range", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
    )
)]
pub async fn rotate_client_secret(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<RotateClientSecretRequest>,
) -> Result<Response<RotateClientSecretResponse>, ApiError> {
    let rotated = state
        .service
        .rotate_client_secret(
            identity,
            RotateClientSecretInput {
                client_id,
                realm_name,
                grace_period_secs: payload.grace_period_secs,
            },
        )
        .await?;

    Ok(Response::OK(RotateClientSecretResponse {
        client_secret: rotated.client_secret,
        previous_secret_expires_at: rotated.previous_secret_expires_at,
    }))
}

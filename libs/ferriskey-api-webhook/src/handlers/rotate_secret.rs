use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::ports::{RotateWebhookSecretInput, WebhookService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct RotateSecretResponse {
    pub secret: String,
}

#[utoipa::path(
    post,
    path = "/{webhook_id}/secret/rotate",
    tag = "webhook",
    summary = "Rotate a webhook's signing secret",
    description = "Generates a new signing secret and returns it. This is the only response that ever contains the secret: it cannot be read again afterwards. Deliveries already signed with the previous secret will fail verification.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("webhook_id" = Uuid, Path, description = "Webhook ID"),
    ),
    responses(
        (status = 200, description = "A new secret was generated and is returned once", body = RotateSecretResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Webhook not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn rotate_secret(
    Path((realm_name, webhook_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<RotateSecretResponse>, ApiError> {
    let secret = state
        .service
        .rotate_webhook_secret(
            identity,
            RotateWebhookSecretInput {
                realm_name,
                webhook_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(RotateSecretResponse { secret }))
}

use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::app_state::AppState;
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::webhook::use_cases::delete_webhook_use_case::DeleteWebhookUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::Deserialize;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/webhooks/{webhook_id}")]
pub struct DeleteWebhookRoute {
    realm_name: String,
    webhook_id: Uuid,
}

#[utoipa::path(
    delete,
    path = "",
    tag = "webhook",
    summary = "Delete webhook",
    description = "Deletes a webhook in the system related to the current realm.",
    responses(
        (status = 200, body = ())
    ),
)]

pub async fn delete_webhook(
    DeleteWebhookRoute {
        realm_name,
        webhook_id,
    }: DeleteWebhookRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<(), ApiError> {
    state
        .use_case_bundle
        .delete_webhook_use_case
        .execute(
            identity,
            DeleteWebhookUseCaseParams {
                realm_name,
                webhook_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(())
}

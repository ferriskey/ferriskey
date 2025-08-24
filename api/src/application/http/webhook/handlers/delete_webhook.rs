use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::webhook::use_cases::delete_webhook_use_case::DeleteWebhookUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

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
    Path(realm_name): Path<String>,
    Path(webhook_id): Path<Uuid>,
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

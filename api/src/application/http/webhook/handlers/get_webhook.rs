use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::domain::realm::ports::RealmService;
use ferriskey_core::domain::webhook::entities::Webhook;
use ferriskey_core::domain::webhook::ports::WebhookService;
use serde::Deserialize;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/webhooks/{webhook_id}")]
pub struct GetWebhookRoute {
    realm_name: String,
    webhook_id: Uuid,
}

#[utoipa::path(
    get,
    path = "/{webhook_id}",
    tag = "webhook",
    summary = "Get webhook",
    description = "Retrieves one webhook in the system related to the current realm.",
    responses(
        (status = 200, body = Webhook)
    ),
)]

pub async fn get_webhook(
    GetWebhookRoute {
        realm_name,
        webhook_id,
    }: GetWebhookRoute,
    State(state): State<AppState>,
) -> Result<Response<Option<Webhook>>, ApiError> {
    let realm = state
        .service_bundle
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let webhook = state
        .service_bundle
        .webhook_service
        .get_by_id(realm.id, webhook_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(webhook))
}

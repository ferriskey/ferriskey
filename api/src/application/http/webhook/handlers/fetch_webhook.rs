use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::domain::realm::ports::RealmService;
use ferriskey_core::domain::webhook::entities::Webhook;
use ferriskey_core::domain::webhook::ports::WebhookService;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/webhooks")]
pub struct FetchWebhookRoute {
    realm_name: String,
}

#[utoipa::path(
    get,
    path = "",
    tag = "webhook",
    summary = "Fetch all webhooks",
    description = "Retrieves a list of all webhooks available in the system related to the current realm.",
    responses(
        (status = 200, body = Vec<Webhook>)
    ),
)]

pub async fn fetch_webhooks(
    FetchWebhookRoute { realm_name }: FetchWebhookRoute,
    State(state): State<AppState>,
) -> Result<Response<Vec<Webhook>>, ApiError> {
    let realm = state
        .service_bundle
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let webhooks = state
        .service_bundle
        .webhook_service
        .fetch_by_realm(realm.id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(webhooks))
}

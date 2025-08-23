use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::webhook::use_cases::fetch_realm_webhooks_use_case::FetchRealmWebhooksUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::entities::webhook::Webhook;
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
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<Webhook>>, ApiError> {
    let webhooks = state
        .use_case_bundle
        .fetch_realm_webhooks_use_case
        .execute(identity, FetchRealmWebhooksUseCaseParams { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(webhooks))
}

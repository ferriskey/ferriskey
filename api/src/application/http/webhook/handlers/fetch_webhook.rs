use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::webhook::use_cases::fetch_realm_webhooks_use_case::FetchRealmWebhooksUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::webhook::entities::webhook::Webhook;

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
    Path(realm_name): Path<String>,
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

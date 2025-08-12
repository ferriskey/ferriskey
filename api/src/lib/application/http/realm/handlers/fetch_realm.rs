use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::domain::realm::entities::Realm;
use ferriskey_core::domain::realm::ports::RealmService;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms")]
pub struct GetRealmRoute {}

#[utoipa::path(
    get,
    path = "",
    tag = "realm",
    summary = "Fetch all realms",
    description = "Retrieves a list of all realms available in the system.",
    responses(
        (status = 200, body = Vec<Realm>)
    ),
)]

pub async fn fetch_realm(
    _: GetRealmRoute,
    State(state): State<AppState>,
) -> Result<Response<Vec<Realm>>, ApiError> {
    let realms = state
        .service_bundle
        .realm_service
        .fetch_realm()
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(realms))
}

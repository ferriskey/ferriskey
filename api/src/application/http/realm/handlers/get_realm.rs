use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::realm::entities::Realm;
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    realm::ports::{GetRealmInput, RealmService},
};

#[utoipa::path(
    get,
    path = "/{name}",
    tag = "realm",
    summary = "Get a realm by name",
    description = "Retrieves a realm by its name. This endpoint returns the details of the specified realm.",
    params(
        ("name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = Realm)
    ),
)]
pub async fn get_realm(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Realm>, ApiError> {
    state
        .service
        .get_realm_by_name(identity, GetRealmInput { realm_name: name })
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

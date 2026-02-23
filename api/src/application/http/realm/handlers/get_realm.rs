use crate::application::http::server::api_entities::api_error::{ApiError, ApiErrorResponse};
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
        (status = 200, description = "Realm retrieved successfully", body = Realm),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
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

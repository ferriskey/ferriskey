use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::realm::use_cases::get_realm_use_case::GetRealmUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::realm::entities::Realm;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{name}")]
pub struct GetRealmRoute {
    pub name: String,
}

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
    GetRealmRoute { name }: GetRealmRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Realm>, ApiError> {
    state
        .use_case_bundle
        .get_realm_use_case
        .execute(identity, GetRealmUseCaseParams { realm_name: name })
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}

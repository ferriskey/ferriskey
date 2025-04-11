use crate::application::http::realm::validators::UpdateRealmValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::domain::realm::{entities::realm::Realm, ports::realm_service::RealmService};
use axum::extract::State;
use axum_macros::TypedPath;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{name}")]
pub struct UpdateRealmRoute {
    pub name: String,
}

#[utoipa::path(
    put,
    path = "/{name}",
    tag = "realm",
    params(
        ("name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = Realm)
    ),
    request_body = UpdateRealmValidator
)]
pub async fn update_realm(
    UpdateRealmRoute { name }: UpdateRealmRoute,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<UpdateRealmValidator>,
) -> Result<Response<Realm>, ApiError> {
    state
        .realm_service
        .update_realm(name, payload.name)
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

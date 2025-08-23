use crate::application::http::server::api_entities::{api_error::ApiError, response::Response};
use crate::application::http::server::app_state::AppState;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::realm::use_cases::get_user_realms_use_case::GetUserRealmsUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::realm::entities::Realm;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UserRealmsResponse {
    pub data: Vec<Realm>,
}

#[utoipa::path(
    get,
    summary = "Get user realms",
    path = "/{realm_name}/users/@me/realms",
    tag = "realm",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    security(
        ("Authorization" = ["Bearer"]),
    ),
    responses(
        (status = 200, body = UserRealmsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn get_user_realms(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UserRealmsResponse>, ApiError> {
    let realms = state
        .use_case_bundle
        .get_user_realms_use_case
        .execute(identity, GetUserRealmsUseCaseParams { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UserRealmsResponse { data: realms }))
}

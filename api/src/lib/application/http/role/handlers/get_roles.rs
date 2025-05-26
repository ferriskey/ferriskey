use axum::extract::State;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

use crate::{
    application::http::server::{
            api_entities::{api_error::ApiError, response::Response},
            app_state::AppState,
        },
    domain::{
        realm::ports::realm_service::RealmService,
        role::{
            entities::models::Role,
            ports::RoleService,
        },
    },
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/roles")]
pub struct GetRolesRoute {
    pub realm_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct GetRolesResponse {
    pub data: Vec<Role>,
}

#[utoipa::path(
    get,
    summary = "Get all roles for a realm",
    path = "/realms/{realm_name}/roles",
    tag = "role",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = GetRolesResponse)
    )
)]
pub async fn get_roles(
    GetRolesRoute { realm_name }: GetRolesRoute,
    State(state): State<AppState>,
) -> Result<Response<GetRolesResponse>, ApiError> {
    let realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let roles = state
        .role_service
        .get_by_realm_id(realm.id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetRolesResponse { data: roles }))
}

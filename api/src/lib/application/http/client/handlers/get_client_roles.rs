use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::domain::role::entities::models::Role;
use crate::domain::role::ports::RoleService;
use axum::extract::State;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients/{client_id}/roles")]
pub struct GetClientRolesRoute {
    pub realm_name: String,
    pub client_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetClientRolesResponse {
    pub data: Vec<Role>,
}

#[utoipa::path(
    get,
    path = "/{client_id}/roles",
    tag = "client",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved client roles", body = GetClientRolesResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_client_roles(
    GetClientRolesRoute {
        realm_name: _,
        client_id,
    }: GetClientRolesRoute,
    State(state): State<AppState>,
) -> Result<Response<GetClientRolesResponse>, ApiError> {
    let roles = state.role_service.get_by_client_id(client_id).await?;

    Ok(Response::OK(GetClientRolesResponse { data: roles }))
}

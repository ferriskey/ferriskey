use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::Extension;
use axum::extract::{Path, State};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::GetClientRolesInput;
use ferriskey_core::domain::client::ports::ClientService;
use ferriskey_core::domain::role::entities::Role;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetClientRolesResponse {
    pub data: Vec<Role>,
}

#[utoipa::path(
    get,
    path = "/{client_id}/roles",
    tag = "client",
    summary = "Get client roles",
    description = "Retrieves all roles associated with a client in a specific realm.",
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
    Path(realm_name): Path<String>,
    Path(client_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetClientRolesResponse>, ApiError> {
    let roles = state
        .service
        .get_client_roles(
            identity,
            GetClientRolesInput {
                client_id,
                realm_name,
            },
        )
        .await?;

    Ok(Response::OK(GetClientRolesResponse { data: roles }))
}

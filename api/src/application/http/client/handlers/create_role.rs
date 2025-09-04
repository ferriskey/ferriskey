use crate::application::http::{
    role::validators::CreateRoleValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::client::ports::ClientService;
use ferriskey_core::domain::role::entities::Role;
use ferriskey_core::domain::{
    authentication::value_objects::Identity, client::entities::CreateRoleInput,
};
use uuid::Uuid;

#[utoipa::path(
    post,
    summary = "Create a new role",
    description = "Creates a new role for a specific client within a realm. This endpoint allows you to define roles that can be assigned to users or groups in the context of a client application.",
    path = "/{client_id}/roles",
    tag = "client",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    request_body = CreateRoleValidator,
    responses(
        (status = 201, body = Role)
    )
)]
pub async fn create_role(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateRoleValidator>,
) -> Result<Response<Role>, ApiError> {
    let role = state
        .service
        .create_role(
            identity,
            CreateRoleInput {
                client_id,
                description: payload.description,
                name: payload.name,
                permissions: payload.permissions,
                realm_name,
            },
        )
        .await?;

    Ok(Response::Created(role))
}

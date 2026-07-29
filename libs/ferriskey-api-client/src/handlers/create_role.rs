use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_role::validators::CreateRoleValidator;
use ferriskey_core::domain::client::ports::ClientService;
use ferriskey_core::domain::role::entities::Role;
use ferriskey_core::domain::{
    authentication::value_objects::Identity, client::entities::CreateRoleInput,
};
use uuid::Uuid;

#[utoipa::path(
    post,
    operation_id = "create_client_role",
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
        (status = 201, body = Role, description = "Role created successfully"),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
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

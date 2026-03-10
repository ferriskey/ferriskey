use crate::application::http::{
    role::validators::CreateRoleValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    role::{
        entities::{CreateRoleInput, Role},
        ports::RoleService,
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CreateRoleResponse {
    pub data: Role,
}

#[utoipa::path(
    post,
    operation_id = "create_realm_role",
    summary = "Create a new realm role",
    description = "Creates a new realm-scoped role in the specified realm.",
    path = "",
    tag = "role",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = CreateRoleValidator,
    responses(
        (status = 201, description = "Realm role created successfully", body = CreateRoleResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn create_role(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateRoleValidator>,
) -> Result<Response<CreateRoleResponse>, ApiError> {
    let role = state
        .service
        .create_role(
            identity,
            CreateRoleInput {
                realm_name,
                name: payload.name,
                description: payload.description,
                permissions: payload.permissions,
            },
        )
        .await?;

    Ok(Response::Created(CreateRoleResponse { data: role }))
}

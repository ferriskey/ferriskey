use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::api_error::{
    ApiError, ApiErrorBody, ApiErrorResponse, ValidateJson,
};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_authz::tenant::ports::AuthzManagementService;
use ferriskey_core::domain::authentication::value_objects::Identity;

use crate::management::{BuiltinSchemaBody, ReplaceSchemaValidator, SchemaBody};

#[utoipa::path(
    get,
    path = "/authz/schema",
    tag = "authz",
    summary = "Get the Cedar schema of a realm",
    description = "The realm's own schema, without the built-in FerrisKey types. Requires view_authorization.",
    params(("realm_name" = String, Path, description = "Realm name")),
    responses(
        (status = 200, description = "Schema", body = SchemaBody),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found, or no schema stored", body = ApiErrorResponse),
    ),
)]
pub async fn get_schema(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SchemaBody>, ApiError> {
    let schema = state
        .service
        .get_schema(identity, realm_name)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(ApiErrorBody::new(
                "This realm has no authorization schema",
                "not_found",
            ))
        })?;
    Ok(Response::OK(schema.into()))
}

#[utoipa::path(
    put,
    path = "/authz/schema",
    tag = "authz",
    summary = "Replace the Cedar schema of a realm",
    description = "Refused when an enabled policy or template no longer validates against the new schema. Requires manage_authorization.",
    params(("realm_name" = String, Path, description = "Realm name")),
    request_body = ReplaceSchemaValidator,
    responses(
        (status = 200, description = "Schema stored", body = SchemaBody),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 409, description = "The rules changed since they were read", body = ApiErrorResponse),
        (status = 422, description = "Schema or dependent policies invalid", body = ApiErrorResponse),
    ),
)]
pub async fn put_schema(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ReplaceSchemaValidator>,
) -> Result<Response<SchemaBody>, ApiError> {
    let input = payload.into_input(realm_name)?;
    let schema = state.service.replace_schema(identity, input).await?;
    Ok(Response::Updated(schema.into()))
}

#[utoipa::path(
    get,
    path = "/authz/schema/builtin",
    tag = "authz",
    summary = "Get the built-in Cedar schema",
    description = "The FerrisKey namespace every realm schema is merged with: users, groups, roles, organizations, and the agent types and actions (call_tool, read_resource, get_prompt). Validate policies client-side against this plus the realm schema. Requires view_authorization.",
    params(("realm_name" = String, Path, description = "Realm name")),
    responses(
        (status = 200, description = "Built-in schema", body = BuiltinSchemaBody),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
    ),
)]
pub async fn get_builtin_schema(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<BuiltinSchemaBody>, ApiError> {
    let source = state
        .service
        .get_builtin_schema(identity, realm_name)
        .await?;
    Ok(Response::OK(BuiltinSchemaBody {
        namespace: "FerrisKey".to_string(),
        source: source.to_string(),
    }))
}

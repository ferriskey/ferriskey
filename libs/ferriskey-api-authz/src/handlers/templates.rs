use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_authz::tenant::ports::AuthzManagementService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

use crate::management::{TemplateResponse, TemplateValidator};

#[utoipa::path(
    get,
    path = "/authz/templates",
    tag = "authz",
    summary = "List the policy templates of a realm",
    description = "Requires view_authorization.",
    params(("realm_name" = String, Path, description = "Realm name")),
    responses(
        (status = 200, description = "Templates", body = Vec<TemplateResponse>),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
    ),
)]
pub async fn list_templates(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<TemplateResponse>>, ApiError> {
    let templates = state.service.list_templates(identity, realm_name).await?;
    Ok(Response::OK(
        templates.into_iter().map(TemplateResponse::from).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/authz/templates",
    tag = "authz",
    summary = "Create a policy template",
    description = "Requires manage_authorization.",
    params(("realm_name" = String, Path, description = "Realm name")),
    request_body = TemplateValidator,
    responses(
        (status = 201, description = "Template created", body = TemplateResponse),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 409, description = "Name taken, or the rules changed since they were read", body = ApiErrorResponse),
        (status = 422, description = "Template invalid", body = ApiErrorResponse),
    ),
)]
pub async fn create_template(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<TemplateValidator>,
) -> Result<Response<TemplateResponse>, ApiError> {
    let input = payload.into_create(realm_name)?;
    let template = state.service.create_template(identity, input).await?;
    Ok(Response::Created(template.into()))
}

#[utoipa::path(
    get,
    path = "/authz/templates/{template_id}",
    tag = "authz",
    summary = "Get a policy template",
    description = "Requires view_authorization.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("template_id" = Uuid, Path, description = "Template ID"),
    ),
    responses(
        (status = 200, description = "Template", body = TemplateResponse),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
    ),
)]
pub async fn get_template(
    Path((realm_name, template_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<TemplateResponse>, ApiError> {
    let template = state
        .service
        .list_templates(identity, realm_name)
        .await?
        .into_iter()
        .find(|template| template.id == template_id)
        .ok_or(ferriskey_domain::common::app_errors::CoreError::NotFound)?;
    Ok(Response::OK(template.into()))
}

#[utoipa::path(
    put,
    path = "/authz/templates/{template_id}",
    tag = "authz",
    summary = "Replace a policy template",
    description = "Policies linked to the template are validated again. Requires manage_authorization.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("template_id" = Uuid, Path, description = "Template ID"),
    ),
    request_body = TemplateValidator,
    responses(
        (status = 200, description = "Template replaced", body = TemplateResponse),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 409, description = "Name taken, or the rules changed since they were read", body = ApiErrorResponse),
        (status = 422, description = "Template invalid", body = ApiErrorResponse),
    ),
)]
pub async fn update_template(
    Path((realm_name, template_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<TemplateValidator>,
) -> Result<Response<TemplateResponse>, ApiError> {
    let input = payload.into_update(realm_name, template_id)?;
    let template = state.service.update_template(identity, input).await?;
    Ok(Response::Updated(template.into()))
}

#[utoipa::path(
    delete,
    path = "/authz/templates/{template_id}",
    tag = "authz",
    summary = "Delete a policy template",
    description = "Refused while a policy links it. Requires manage_authorization.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("template_id" = Uuid, Path, description = "Template ID"),
    ),
    responses(
        (status = 204, description = "Template deleted"),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 409, description = "Still linked, or the rules changed since they were read", body = ApiErrorResponse),
    ),
)]
pub async fn delete_template(
    Path((realm_name, template_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .delete_template(identity, realm_name, template_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

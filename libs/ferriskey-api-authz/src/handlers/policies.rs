use axum::{
    Extension,
    extract::{Path, Query, State},
    http::StatusCode,
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_authz::tenant::ports::AuthzManagementService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

use crate::management::{
    ManagedByQuery, PolicyResponse, PolicyValidator, ValidatePoliciesValidator, ValidationResponse,
};

#[utoipa::path(
    get,
    path = "/authz/policies",
    tag = "authz",
    summary = "List the policies of a realm",
    description = "With managed_by, only the policies carrying that label. Requires view_authorization.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("managed_by" = Option<String>, Query, description = "Only the policies labelled with this manager"),
    ),
    responses(
        (status = 200, description = "Policies", body = Vec<PolicyResponse>),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
    ),
)]
pub async fn list_policies(
    Path(realm_name): Path<String>,
    Query(query): Query<ManagedByQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<PolicyResponse>>, ApiError> {
    let policies = state
        .service
        .list_policies(identity, realm_name, query.manager()?)
        .await?;
    Ok(Response::OK(
        policies.into_iter().map(PolicyResponse::from).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/authz/policies",
    tag = "authz",
    summary = "Create a policy",
    description = "The policy is validated with every other enabled policy against the realm's schema. With managed_by, the policy is labelled with that manager. Requires manage_authorization.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("managed_by" = Option<String>, Query, description = "Label the policy with this manager"),
    ),
    request_body = PolicyValidator,
    responses(
        (status = 201, description = "Policy created", body = PolicyResponse),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 409, description = "Name taken, or the rules changed since they were read", body = ApiErrorResponse),
        (status = 422, description = "Policy invalid", body = ApiErrorResponse),
    ),
)]
pub async fn create_policy(
    Path(realm_name): Path<String>,
    Query(query): Query<ManagedByQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<PolicyValidator>,
) -> Result<Response<PolicyResponse>, ApiError> {
    let input = payload.into_create(realm_name, query.manager()?)?;
    let policy = state.service.create_policy(identity, input).await?;
    Ok(Response::Created(policy.into()))
}

#[utoipa::path(
    get,
    path = "/authz/policies/{policy_id}",
    tag = "authz",
    summary = "Get a policy",
    description = "With managed_by, a policy of another manager or without a label is not found. Requires view_authorization.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("policy_id" = Uuid, Path, description = "Policy ID"),
        ("managed_by" = Option<String>, Query, description = "Only a policy labelled with this manager"),
    ),
    responses(
        (status = 200, description = "Policy", body = PolicyResponse),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
    ),
)]
pub async fn get_policy(
    Path((realm_name, policy_id)): Path<(String, Uuid)>,
    Query(query): Query<ManagedByQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<PolicyResponse>, ApiError> {
    let policy = state
        .service
        .get_policy(identity, realm_name, policy_id, query.manager()?)
        .await?;
    Ok(Response::OK(policy.into()))
}

#[utoipa::path(
    put,
    path = "/authz/policies/{policy_id}",
    tag = "authz",
    summary = "Replace a policy",
    description = "With managed_by, only a policy already labelled with that manager can be replaced; without it the existing label is kept. Requires manage_authorization.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("policy_id" = Uuid, Path, description = "Policy ID"),
        ("managed_by" = Option<String>, Query, description = "The manager replacing its own policy"),
    ),
    request_body = PolicyValidator,
    responses(
        (status = 200, description = "Policy replaced", body = PolicyResponse),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 409, description = "Name taken, or the rules changed since they were read", body = ApiErrorResponse),
        (status = 422, description = "Policy invalid", body = ApiErrorResponse),
    ),
)]
pub async fn update_policy(
    Path((realm_name, policy_id)): Path<(String, Uuid)>,
    Query(query): Query<ManagedByQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<PolicyValidator>,
) -> Result<Response<PolicyResponse>, ApiError> {
    let input = payload.into_update(realm_name, policy_id, query.manager()?)?;
    let policy = state.service.update_policy(identity, input).await?;
    Ok(Response::Updated(policy.into()))
}

#[utoipa::path(
    delete,
    path = "/authz/policies/{policy_id}",
    tag = "authz",
    summary = "Delete a policy",
    description = "With managed_by, only a policy labelled with that manager can be deleted. Requires manage_authorization.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("policy_id" = Uuid, Path, description = "Policy ID"),
        ("managed_by" = Option<String>, Query, description = "The manager deleting its own policy"),
    ),
    responses(
        (status = 204, description = "Policy deleted"),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Not found", body = ApiErrorResponse),
        (status = 409, description = "The rules changed since they were read", body = ApiErrorResponse),
    ),
)]
pub async fn delete_policy(
    Path((realm_name, policy_id)): Path<(String, Uuid)>,
    Query(query): Query<ManagedByQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .delete_policy(identity, realm_name, policy_id, query.manager()?)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/authz/policies/validate",
    tag = "authz",
    summary = "Validate policies without saving them",
    description = "Parses the Cedar and validates it against the realm's schema. Problems come back as diagnostics with line and column, in a 200; nothing is stored. Requires view_authorization.",
    params(("realm_name" = String, Path, description = "Realm name")),
    request_body = ValidatePoliciesValidator,
    responses(
        (status = 200, description = "Diagnostics", body = ValidationResponse),
        (status = 400, description = "Malformed request", body = ApiErrorResponse),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
    ),
)]
pub async fn validate_policies(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ValidatePoliciesValidator>,
) -> Result<Response<ValidationResponse>, ApiError> {
    let input = payload.into_input(realm_name)?;
    let issues = state.service.validate_policies(identity, input).await?;
    Ok(Response::OK(issues.into()))
}

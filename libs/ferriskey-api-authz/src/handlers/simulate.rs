use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_authz::tenant::ports::AuthzManagementService;
use ferriskey_core::domain::authentication::value_objects::Identity;

use crate::management::SimulateValidator;
use crate::validators::EvaluationResponse;

#[utoipa::path(
    post,
    path = "/authz/simulate",
    tag = "authz",
    summary = "Evaluate a request against a policy set without saving it",
    description = "Dry run. The policies in the body, or the realm's enabled ones when absent, are validated and then decide the request. FerrisKey subjects are not looked up: describe them, and their groups and roles as parents, in entities. Requires view_authorization.",
    params(("realm_name" = String, Path, description = "Realm name")),
    request_body = SimulateValidator,
    responses(
        (status = 200, description = "Decision, with the names of the determining policies", body = EvaluationResponse),
        (status = 400, description = "Request not valid against the schema", body = ApiErrorResponse),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
        (status = 422, description = "Policies invalid", body = ApiErrorResponse),
    ),
)]
pub async fn simulate(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<SimulateValidator>,
) -> Result<Response<EvaluationResponse>, ApiError> {
    let input = payload.into_input(realm_name)?;
    let outcome = state.service.simulate(identity, input).await?;
    Ok(Response::OK(outcome.into()))
}

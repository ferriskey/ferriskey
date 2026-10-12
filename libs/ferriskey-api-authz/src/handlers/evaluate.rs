use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_authz::tenant::evaluation::EvaluationRequest;
use ferriskey_authz::tenant::ports::EvaluationService;
use ferriskey_core::domain::authentication::value_objects::Identity;

use crate::validators::{DecisionContext, EvaluationResponse, EvaluationValidator, ReasonAdmin};

#[utoipa::path(
    post,
    path = "/authzen/access/v1/evaluation",
    tag = "authzen",
    summary = "Evaluate an access request",
    description = "AuthZEN 1.0 access evaluation. Called by a policy enforcement point authenticated as a service account holding evaluate_authorization. A refusal is a 200 with decision false; any other status carries no decision.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = EvaluationValidator,
    responses(
        (status = 200, description = "Decision", body = EvaluationResponse),
        (status = 400, description = "Invalid request", body = ApiErrorResponse),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Authorization not enabled for this realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn evaluate(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<EvaluationValidator>,
) -> Result<Response<EvaluationResponse>, ApiError> {
    let request = EvaluationRequest::try_from(payload)?;

    let outcome = state
        .service
        .evaluate(identity, realm_name, request)
        .await?;

    Ok(Response::OK(EvaluationResponse {
        decision: outcome.allowed,
        context: outcome.reason_admin.map(|policies| DecisionContext {
            reason_admin: ReasonAdmin { policies },
        }),
    }))
}

use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_authz::tenant::ports::EvaluationService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde_json::Value;

use crate::validators::{
    BatchDecision, DecisionContext, EvaluationResponse, EvaluationsResponse, EvaluationsValidator,
    ReasonAdmin,
};

#[utoipa::path(
    post,
    path = "/authzen/access/v1/evaluations",
    tag = "authzen",
    summary = "Evaluate several access requests",
    description = "AuthZEN 1.0 boxcarring. Top-level subject, action, resource and context are defaults each item may override. options.evaluations_semantic is execute_all (default), deny_on_first_deny or permit_on_first_permit; a short-circuited response holds the items evaluated so far, in request order. An item the realm's schema rejects is a deny whose context.error says why. Without evaluations the request is a single evaluation. At most 100 items. Called by a service account holding evaluate_authorization.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = EvaluationsValidator,
    responses(
        (status = 200, description = "Decisions", body = EvaluationsResponse),
        (status = 400, description = "Invalid request", body = ApiErrorResponse),
        (status = 401, description = "Missing or invalid token", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Authorization not enabled for this realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn evaluations(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<EvaluationsValidator>,
) -> Result<Response<Value>, ApiError> {
    let Some(requests) = payload.requests()? else {
        let outcome = state
            .service
            .evaluate(identity, realm_name, payload.single()?)
            .await?;
        let single = EvaluationResponse {
            decision: outcome.allowed,
            context: outcome.reason_admin.map(|policies| DecisionContext {
                reason_admin: ReasonAdmin { policies },
            }),
        };
        return Ok(Response::OK(to_value(&single)?));
    };

    let outcomes = state
        .service
        .evaluate_many(identity, realm_name, requests, payload.semantic())
        .await?;
    let body = EvaluationsResponse {
        evaluations: outcomes.into_iter().map(BatchDecision::from).collect(),
    };
    Ok(Response::OK(to_value(&body)?))
}

fn to_value<T: serde::Serialize>(value: &T) -> Result<Value, ApiError> {
    serde_json::to_value(value).map_err(|_| {
        ApiError::InternalServerError(
            ferriskey_api_core::api_entities::api_error::ApiErrorBody::new(
                "could not encode the response",
                "internal_server_error",
            ),
        )
    })
}

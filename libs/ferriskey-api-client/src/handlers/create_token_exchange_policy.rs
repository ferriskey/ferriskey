use crate::validators::CreateTokenExchangePolicyValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::token_exchange_policy::TokenExchangePolicy;
use ferriskey_core::domain::client::value_objects::CreateTokenExchangePolicyRequest;
use ferriskey_core::domain::client::{
    entities::CreateTokenExchangePolicyInput, ports::TokenExchangePolicyService,
};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/{client_id}/token-exchange-policies",
    summary = "Allow a client to exchange tokens for another audience",
    description = "Registers an RFC 8693 token-exchange delegation policy. `target_audience` is the `client_id` of another client of the same realm. `allowed_scopes`, when set, caps the scopes the exchanged token may carry; each entry is a single scope token without whitespace.",
    responses(
        (status = 201, body = TokenExchangePolicy, description = "Token exchange policy created"),
        (status = 400, description = "The target audience is not another client of this realm, or a scope is malformed", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
        (status = 409, description = "A policy already targets this audience for this client", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    request_body = CreateTokenExchangePolicyValidator,
)]
pub async fn create_token_exchange_policy(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateTokenExchangePolicyValidator>,
) -> Result<Response<TokenExchangePolicy>, ApiError> {
    state
        .service
        .create_token_exchange_policy(
            identity,
            CreateTokenExchangePolicyInput {
                realm_name,
                client_id,
                payload: CreateTokenExchangePolicyRequest {
                    target_audience: payload.target_audience,
                    allowed_scopes: payload.allowed_scopes,
                    allow_impersonation: payload.allow_impersonation,
                    allow_delegation: payload.allow_delegation,
                },
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

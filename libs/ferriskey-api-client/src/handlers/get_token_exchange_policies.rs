use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::token_exchange_policy::TokenExchangePolicy;
use ferriskey_core::domain::client::{
    entities::GetTokenExchangePoliciesInput, ports::TokenExchangePolicyService,
};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{client_id}/token-exchange-policies",
    summary = "List the token exchange policies of a client",
    description = "Returns the RFC 8693 token-exchange delegation policies registered for this client, one per target audience.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, body = Vec<TokenExchangePolicy>),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_token_exchange_policies(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<TokenExchangePolicy>>, ApiError> {
    state
        .service
        .get_token_exchange_policies(
            identity,
            GetTokenExchangePoliciesInput {
                realm_name,
                client_id,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}

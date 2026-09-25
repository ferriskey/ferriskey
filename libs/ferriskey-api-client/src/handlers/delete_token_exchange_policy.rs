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
use ferriskey_core::domain::client::{
    entities::DeleteTokenExchangePolicyInput, ports::TokenExchangePolicyService,
};
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/{client_id}/token-exchange-policies/{policy_id}",
    summary = "Remove a token exchange policy from a client",
    description = "Removes an RFC 8693 token-exchange delegation policy. Tokens already issued through it stay valid until they expire.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("policy_id" = Uuid, Path, description = "Token exchange policy ID"),
    ),
    tag = "client",
    responses(
        (status = 200, description = "Token exchange policy removed"),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Token exchange policy not found for this client", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_token_exchange_policy(
    Path((realm_name, client_id, policy_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .delete_token_exchange_policy(
            identity,
            DeleteTokenExchangePolicyInput {
                realm_name,
                client_id,
                policy_id,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}

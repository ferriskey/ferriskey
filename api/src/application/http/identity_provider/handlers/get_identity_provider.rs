use crate::application::http::{
    identity_provider::validators::IdentityProviderResponse,
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::identity_provider::{
    entities::GetIdentityProviderInput, ports::IdentityProviderService,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity, identity_provider::IdentityProvider,
};

#[utoipa::path(
    get,
    path = "/{alias}",
    summary = "Get an identity provider by alias",
    description = "Retrieves the details of a specific identity provider by its alias. Sensitive configuration values (like client secrets) are redacted in the response.",
    responses(
        (status = 200, body = IdentityProviderResponse, description = "Identity provider details"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Identity provider or realm not found"),
    ),
    params(
        ("realm_name" = String, Path, description = "The name of the realm"),
        ("alias" = String, Path, description = "The unique alias of the identity provider"),
    ),
    tag = "identity_provider",
)]
pub async fn get_identity_provider(
    Path((realm_name, alias)): Path<(String, String)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<IdentityProvider>, ApiError> {
    let provider = state
        .service
        .get_identity_provider(identity, GetIdentityProviderInput { realm_name, alias })
        .await?;

    Ok(Response::OK(provider))
}

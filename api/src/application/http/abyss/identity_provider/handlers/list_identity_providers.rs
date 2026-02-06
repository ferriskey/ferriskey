use crate::application::http::{
    abyss::identity_provider::dto::{
        IdentityProviderResponse, IdentityProvidersResponse, ListIdentityProvidersQuery,
    },
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, Query, State},
};
use ferriskey_core::domain::abyss::identity_provider::{
    entities::ListIdentityProvidersInput, ports::IdentityProviderService,
};
use ferriskey_core::domain::authentication::value_objects::Identity;

#[utoipa::path(
    get,
    path = "/identity-providers",
    summary = "List all identity providers in a realm",
    description = "Retrieves all identity providers configured for the specified realm. Optionally returns a brief representation with fewer fields.",
    responses(
        (status = 200, body = IdentityProvidersResponse, description = "List of identity providers"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Realm not found"),
    ),
    params(
        ("realm_name" = String, Path, description = "The name of the realm"),
        ListIdentityProvidersQuery,
    ),
    tag = "identity_provider",
)]
pub async fn list_identity_providers(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(_query): Query<ListIdentityProvidersQuery>,
) -> Result<Response<IdentityProvidersResponse>, ApiError> {
    let providers = state
        .service
        .list_identity_providers(identity, ListIdentityProvidersInput { realm_name })
        .await?
        .into_iter()
        .map(|p| p.into())
        .collect::<Vec<IdentityProviderResponse>>();

    Ok(Response::OK(IdentityProvidersResponse { data: providers }))
}

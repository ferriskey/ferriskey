use crate::application::http::{
    identity_provider::validators::{IdentityProvidersResponse, ListIdentityProvidersQuery},
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, Query, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;

#[utoipa::path(
    get,
    path = "",
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
    Path(_realm_name): Path<String>,
    State(_state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Query(_query): Query<ListIdentityProvidersQuery>,
) -> Result<Response<IdentityProvidersResponse>, ApiError> {
    unimplemented!()
}

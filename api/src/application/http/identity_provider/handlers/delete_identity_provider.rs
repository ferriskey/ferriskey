use crate::application::http::{
    identity_provider::validators::DeleteIdentityProviderResponse,
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;

#[utoipa::path(
    delete,
    path = "/{alias}",
    summary = "Delete an identity provider",
    description = "Deletes an identity provider from the realm. This action is irreversible. Users who have linked accounts with this identity provider will no longer be able to use it for authentication.",
    responses(
        (status = 200, body = DeleteIdentityProviderResponse, description = "Identity provider deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Identity provider or realm not found"),
        (status = 409, description = "Identity provider is still in use"),
    ),
    params(
        ("realm_name" = String, Path, description = "The name of the realm"),
        ("alias" = String, Path, description = "The unique alias of the identity provider to delete"),
    ),
    tag = "identity_provider",
)]
pub async fn delete_identity_provider(
    Path((_realm_name, _alias)): Path<(String, String)>,
    State(_state): State<AppState>,
    Extension(_identity): Extension<Identity>,
) -> Result<Response<DeleteIdentityProviderResponse>, ApiError> {
    unimplemented!()
}

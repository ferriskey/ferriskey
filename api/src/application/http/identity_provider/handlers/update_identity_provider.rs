use crate::application::http::{
    identity_provider::validators::UpdateIdentityProviderValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;

#[utoipa::path(
    put,
    path = "/{alias}",
    summary = "Update an identity provider",
    description = "Updates an existing identity provider configuration. Only the fields provided in the request body will be updated. The alias cannot be changed after creation.",
    responses(
        (status = 204, description = "Identity provider updated successfully"),
        (status = 400, description = "Invalid configuration"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Identity provider or realm not found"),
    ),
    params(
        ("realm_name" = String, Path, description = "The name of the realm"),
        ("alias" = String, Path, description = "The unique alias of the identity provider to update"),
    ),
    tag = "identity_provider",
    request_body = UpdateIdentityProviderValidator,
)]
pub async fn update_identity_provider(
    Path((_realm_name, _alias)): Path<(String, String)>,
    State(_state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    ValidateJson(_payload): ValidateJson<UpdateIdentityProviderValidator>,
) -> Result<Response<()>, ApiError> {
    unimplemented!()
}

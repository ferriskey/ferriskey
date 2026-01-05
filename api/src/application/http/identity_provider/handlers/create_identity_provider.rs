use crate::application::http::{
    identity_provider::validators::{CreateIdentityProviderValidator, IdentityProviderResponse},
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
    post,
    path = "",
    summary = "Create a new identity provider in a realm",
    description = "Creates a new identity provider configuration for the specified realm. The identity provider will be used for social login (Google, GitHub, etc.) or OIDC/SAML federation.",
    responses(
        (status = 201, body = IdentityProviderResponse, description = "Identity provider created successfully"),
        (status = 400, description = "Invalid configuration or missing required fields"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Realm not found"),
        (status = 409, description = "Identity provider alias already exists"),
    ),
    params(
        ("realm_name" = String, Path, description = "The name of the realm"),
    ),
    tag = "identity_provider",
    request_body = CreateIdentityProviderValidator,
)]
pub async fn create_identity_provider(
    Path(_realm_name): Path<String>,
    State(_state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    ValidateJson(_payload): ValidateJson<CreateIdentityProviderValidator>,
) -> Result<Response<IdentityProviderResponse>, ApiError> {
    unimplemented!()
}

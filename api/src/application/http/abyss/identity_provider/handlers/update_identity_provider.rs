use crate::application::http::{
    abyss::identity_provider::dto::{IdentityProviderResponse, UpdateIdentityProviderValidator},
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
use ferriskey_core::domain::abyss::identity_provider::{
    entities::UpdateIdentityProviderInput, ports::IdentityProviderService,
};
use ferriskey_core::domain::authentication::value_objects::Identity;

#[utoipa::path(
    put,
    path = "/identity-providers/{alias}",
    summary = "Update an identity provider",
    description = "Updates an existing identity provider configuration. Only the fields provided in the request body will be updated. The alias cannot be changed after creation.",
    responses(
        (status = 200, body = IdentityProviderResponse, description = "Identity provider updated successfully"),
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
    Path((realm_name, alias)): Path<(String, String)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateIdentityProviderValidator>,
) -> Result<Response<IdentityProviderResponse>, ApiError> {
    let provider = state
        .service
        .update_identity_provider(
            identity,
            UpdateIdentityProviderInput {
                realm_name,
                alias,
                enabled: payload.enabled,
                display_name: payload.display_name,
                first_broker_login_flow_alias: payload.first_broker_login_flow_alias,
                post_broker_login_flow_alias: payload.post_broker_login_flow_alias,
                store_token: payload.store_token,
                add_read_token_role_on_create: payload.add_read_token_role_on_create,
                trust_email: payload.trust_email,
                link_only: payload.link_only,
                config: payload.config,
            },
        )
        .await?;

    Ok(Response::OK(provider.into()))
}

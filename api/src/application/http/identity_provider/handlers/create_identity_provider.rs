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
use ferriskey_core::domain::identity_provider::{
    entities::CreateIdentityProviderInput, ports::IdentityProviderService,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity, identity_provider::IdentityProvider,
};

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
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateIdentityProviderValidator>,
) -> Result<Response<IdentityProvider>, ApiError> {
    let provider = state
        .service
        .create_identity_provider(
            identity,
            CreateIdentityProviderInput {
                realm_name,
                alias: payload.alias,
                provider_id: payload.provider_id,
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

    Ok(Response::Created(provider))
}

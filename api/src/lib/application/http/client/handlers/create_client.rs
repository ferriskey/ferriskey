use crate::application::http::{
    client::validators::CreateClientValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::client::use_cases::create_client_use_case::CreateClientUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::Client;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients")]
pub struct CreateClientRoute {
    pub realm_name: String,
}

#[utoipa::path(
    post,
    path = "",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    tag = "client",
    request_body = CreateClientValidator,
)]
pub async fn create_client(
    CreateClientRoute { realm_name }: CreateClientRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateClientValidator>,
) -> Result<Response<Client>, ApiError> {
    let client = state
        .use_case_bundle
        .create_client_use_case
        .execute(
            identity,
            CreateClientUseCaseParams {
                client_id: payload.client_id,
                client_type: payload.client_type,
                public_client: payload.public_client,
                realm_name,
                enabled: payload.enabled,
                name: payload.name,
                protocol: payload.protocol,
                service_account_enabled: payload.service_account_enabled,
            },
        )
        .await?;

    Ok(Response::Created(client))
}

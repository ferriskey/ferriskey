use axum::extract::State;
use axum_macros::TypedPath;
use serde::Deserialize;

use crate::{
    application::http::{
        client::validators::CreateClientValidator,
        server::{
            api_entities::{
                api_error::{ApiError, ValidateJson},
                response::Response,
            },
            app_state::AppState,
        },
    },
    domain::{
        client::{
            entities::{dto::CreateClientDto, model::Client},
            ports::client_service::ClientService,
        },
        realm::ports::realm_service::RealmService,
        utils::generate_random_string,
    },
};

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
    ValidateJson(payload): ValidateJson<CreateClientValidator>,
) -> Result<Response<Client>, ApiError> {
    let realm = state.realm_service.get_by_name(realm_name.clone()).await?;

    state
        .client_service
        .create_client(
            CreateClientDto {
                realm_id: realm.id,
                name: payload.name,
                client_id: payload.client_id,
                secret: generate_random_string(),
                enabled: payload.enabled,
                protocol: payload.protocol,
                public_client: payload.public_client,
                service_account_enabled: payload.service_account_enabled,
                client_type: payload.client_type,
            },
            realm_name,
        )
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}

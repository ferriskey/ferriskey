use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::application::client::use_cases::get_client_use_case::GetClientUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::Client;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetClientResponse {
    pub data: Client,
}

#[utoipa::path(
    get,
    path = "/{client_id}",
    summary = "Get a client",
    description = "Retrieves a client from the specified realm by its ID.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, description = "Client retrieved successfully", body = GetClientResponse),
    )
)]
pub async fn get_client(
    Path(realm_name): Path<String>,
    Path(client_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetClientResponse>, ApiError> {
    let client = state
        .use_case_bundle
        .get_client_use_case
        .execute(
            identity,
            GetClientUseCaseParams {
                client_id,
                realm_name,
            },
        )
        .await?;

    Ok(Response::OK(GetClientResponse { data: client }))
}

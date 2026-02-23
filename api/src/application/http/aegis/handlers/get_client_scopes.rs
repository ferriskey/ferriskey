use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::aegis::entities::ClientScope;
use ferriskey_core::domain::aegis::ports::ClientScopeService;
use ferriskey_core::domain::aegis::value_objects::GetClientScopesInput;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ClientScopesResponse {
    pub data: Vec<ClientScope>,
}

#[utoipa::path(
    get,
    path = "",
    summary = "Get client scopes in a realm",
    description = "Retrieves all client scopes associated with a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    tag = "client-scope",
    responses(
        (status = 200, description = "Client scopes retrieved successfully", body = ClientScopesResponse),
    )
)]
pub async fn get_client_scopes(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ClientScopesResponse>, ApiError> {
    let scopes = state
        .service
        .get_client_scopes(identity, GetClientScopesInput { realm_name })
        .await?;

    Ok(Response::OK(ClientScopesResponse { data: scopes }))
}

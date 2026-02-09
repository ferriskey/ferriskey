use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::aegis::ports::ProtocolMapperService;
use ferriskey_core::domain::aegis::value_objects::DeleteProtocolMapperInput;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteProtocolMapperResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/{scope_id}/protocol-mappers/{mapper_id}",
    summary = "Delete a protocol mapper",
    description = "Deletes a protocol mapper from the specified client scope.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("scope_id" = Uuid, Path, description = "Client scope ID"),
        ("mapper_id" = Uuid, Path, description = "Protocol mapper ID"),
    ),
    tag = "client-scope",
    responses(
        (status = 200, body = DeleteProtocolMapperResponse, description = "Protocol mapper deleted successfully"),
    ),
)]
pub async fn delete_protocol_mapper(
    Path((realm_name, scope_id, mapper_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteProtocolMapperResponse>, ApiError> {
    state
        .service
        .delete_protocol_mapper(
            identity,
            DeleteProtocolMapperInput {
                realm_name,
                scope_id,
                mapper_id,
            },
        )
        .await?;

    Ok(Response::OK(DeleteProtocolMapperResponse {
        message: format!("Protocol mapper with ID {mapper_id} deleted successfully"),
    }))
}

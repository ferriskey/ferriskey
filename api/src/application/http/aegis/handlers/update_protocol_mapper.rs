use crate::application::http::{
    aegis::validators::UpdateProtocolMapperValidator,
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
use ferriskey_core::domain::aegis::entities::ProtocolMapper;
use ferriskey_core::domain::aegis::ports::ProtocolMapperService;
use ferriskey_core::domain::aegis::value_objects::{
    UpdateProtocolMapperInput, UpdateProtocolMapperRequest,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

#[utoipa::path(
    patch,
    path = "/{scope_id}/protocol-mappers/{mapper_id}",
    summary = "Update a protocol mapper",
    description = "Updates an existing protocol mapper for the specified client scope.",
    responses(
        (status = 200, description = "Protocol mapper updated successfully", body = ProtocolMapper),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("scope_id" = Uuid, Path, description = "Client scope ID"),
        ("mapper_id" = Uuid, Path, description = "Protocol mapper ID"),
    ),
    tag = "client-scope",
    request_body = UpdateProtocolMapperValidator,
)]
pub async fn update_protocol_mapper(
    Path((realm_name, scope_id, mapper_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateProtocolMapperValidator>,
) -> Result<Response<ProtocolMapper>, ApiError> {
    state
        .service
        .update_protocol_mapper(
            identity,
            UpdateProtocolMapperInput {
                realm_name,
                scope_id,
                mapper_id,
                payload: UpdateProtocolMapperRequest {
                    name: payload.name,
                    mapper_type: payload.mapper_type,
                    config: payload.config,
                },
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}

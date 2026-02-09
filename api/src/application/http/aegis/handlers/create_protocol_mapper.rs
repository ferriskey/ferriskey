use crate::application::http::{
    aegis::validators::CreateProtocolMapperValidator,
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
use ferriskey_core::domain::aegis::value_objects::CreateProtocolMapperInput;
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/{scope_id}/protocol-mappers",
    summary = "Create a protocol mapper",
    description = "Creates a new protocol mapper for the specified client scope.",
    responses(
        (status = 201, body = ProtocolMapper, description = "Protocol mapper created successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("scope_id" = Uuid, Path, description = "Client scope ID"),
    ),
    tag = "client-scope",
    request_body = CreateProtocolMapperValidator,
)]
pub async fn create_protocol_mapper(
    Path((realm_name, scope_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateProtocolMapperValidator>,
) -> Result<Response<ProtocolMapper>, ApiError> {
    let mapper = state
        .service
        .create_protocol_mapper(
            identity,
            CreateProtocolMapperInput {
                realm_name,
                scope_id,
                name: payload.name,
                mapper_type: payload.mapper_type,
                config: payload.config,
            },
        )
        .await?;

    Ok(Response::Created(mapper))
}

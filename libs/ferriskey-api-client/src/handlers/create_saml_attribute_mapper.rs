use crate::validators::CreateSamlAttributeMapperValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::saml::SamlAttributeMapper;
use ferriskey_core::domain::client::value_objects::CreateSamlAttributeMapperRequest;
use ferriskey_core::domain::client::{
    entities::CreateSamlAttributeMapperInput, ports::ClientService,
};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/{client_id}/saml-attribute-mappers",
    summary = "Map a FerrisKey user value onto a SAML assertion attribute",
    description = "Declares one `<saml:Attribute>` the assertion carries for this client. `name` is the attribute name the service provider reads — Chatwoot reads `email`, `first_name` and `last_name`. `source` names the value it is fed from: one of `user:id`, `user:username`, `user:email`, `user:first_name`, `user:last_name`, or `attribute:<key>` for a custom user attribute. A name may be mapped once per client.",
    responses(
        (status = 201, body = SamlAttributeMapper, description = "Attribute mapper created"),
        (status = 400, description = "The name, name format or source is invalid, or the name is already mapped for this client", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    request_body = CreateSamlAttributeMapperValidator,
)]
pub async fn create_saml_attribute_mapper(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateSamlAttributeMapperValidator>,
) -> Result<Response<SamlAttributeMapper>, ApiError> {
    let mapper = state
        .service
        .create_saml_attribute_mapper(
            identity,
            CreateSamlAttributeMapperInput {
                client_id,
                realm_name,
                payload: CreateSamlAttributeMapperRequest {
                    name: payload.name,
                    name_format: payload.name_format,
                    source: payload.source,
                },
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Created(mapper))
}

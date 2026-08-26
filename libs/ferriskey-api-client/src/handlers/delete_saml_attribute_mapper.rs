use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::{
    entities::DeleteSamlAttributeMapperInput, ports::ClientService,
};
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/{client_id}/saml-attribute-mappers/{mapper_id}",
    summary = "Stop sending a SAML assertion attribute to this client",
    description = "Removes one attribute mapper. The lookup is scoped to the owning client, so a mapper identifier belonging to another client is not found rather than deleted.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("mapper_id" = Uuid, Path, description = "Attribute mapper ID"),
    ),
    tag = "client",
    responses(
        (status = 200, description = "Attribute mapper removed"),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Attribute mapper not found for this client", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_saml_attribute_mapper(
    Path((realm_name, client_id, mapper_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .delete_saml_attribute_mapper(
            identity,
            DeleteSamlAttributeMapperInput {
                client_id,
                mapper_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(()))
}

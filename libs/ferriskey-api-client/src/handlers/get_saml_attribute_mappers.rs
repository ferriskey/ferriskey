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
use ferriskey_core::domain::client::entities::saml::SamlAttributeMapper;
use ferriskey_core::domain::client::{
    entities::GetSamlAttributeMappersInput, ports::ClientService,
};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{client_id}/saml-attribute-mappers",
    summary = "List the SAML assertion attributes this client is sent",
    description = "Returns every `<saml:Attribute>` the assertion carries for this client, ordered by attribute name.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, body = Vec<SamlAttributeMapper>),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_saml_attribute_mappers(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<SamlAttributeMapper>>, ApiError> {
    state
        .service
        .get_saml_attribute_mappers(
            identity,
            GetSamlAttributeMappersInput {
                client_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}

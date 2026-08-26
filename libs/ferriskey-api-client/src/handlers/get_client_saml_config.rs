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
use ferriskey_core::domain::client::entities::saml::ClientSamlConfig;
use ferriskey_core::domain::client::{entities::GetClientSamlConfigInput, ports::ClientService};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{client_id}/saml-config",
    summary = "Read the SAML service provider configuration of a client",
    description = "Returns the service provider this client stands for: its EntityID, the assertion consumer service URL assertions are posted to, the NameID format identifying the subject, and the signing options. A client that has never been configured for SAML answers 404.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, body = ClientSamlConfig),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Client not found, or the client carries no SAML configuration", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_client_saml_config(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ClientSamlConfig>, ApiError> {
    state
        .service
        .get_client_saml_config(
            identity,
            GetClientSamlConfigInput {
                client_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}

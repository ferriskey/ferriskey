use crate::validators::SetClientSamlConfigValidator;
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
use ferriskey_core::domain::client::entities::saml::ClientSamlConfig;
use ferriskey_core::domain::client::value_objects::SetClientSamlConfigRequest;
use ferriskey_core::domain::client::{entities::SetClientSamlConfigInput, ports::ClientService};
use uuid::Uuid;

#[utoipa::path(
    put,
    path = "/{client_id}/saml-config",
    summary = "Write the SAML service provider configuration of a client",
    description = "Declares the service provider this client stands for. A client holds at most one SAML configuration, so this replaces it wholesale rather than creating a second one. The EntityID must be an absolute URI and is stored byte-for-byte, since SAML compares it verbatim against the issuer of an incoming AuthnRequest. The assertion consumer service URL must be an absolute http or https URL carrying no credentials and no fragment; it may carry a query string, which is how Chatwoot passes its account id.",
    responses(
        (status = 201, body = ClientSamlConfig, description = "SAML configuration written"),
        (status = 400, description = "The entity id, the acs url or the name id format is invalid", body = ApiErrorResponse),
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
    request_body = SetClientSamlConfigValidator,
)]
pub async fn set_client_saml_config(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<SetClientSamlConfigValidator>,
) -> Result<Response<ClientSamlConfig>, ApiError> {
    let config = state
        .service
        .set_client_saml_config(
            identity,
            SetClientSamlConfigInput {
                client_id,
                realm_name,
                payload: SetClientSamlConfigRequest {
                    sp_entity_id: payload.sp_entity_id,
                    acs_url: payload.acs_url,
                    name_id_format: payload.name_id_format,
                    sign_assertions: payload.sign_assertions,
                    sign_documents: payload.sign_documents,
                    want_authn_requests_signed: payload.want_authn_requests_signed,
                },
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Created(config))
}

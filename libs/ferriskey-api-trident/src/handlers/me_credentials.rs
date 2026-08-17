use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
    auth::AuthenticatedRealm,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity, credential::entities::CredentialOverview,
    trident::ports::TridentService,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct MeCredentialsResponse {
    pub data: Vec<CredentialOverview>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MeDeleteCredentialResponse {
    pub message: String,
}

/// Request body for deleting a credential. Carries the step-up token minted by
/// `/me/reauthenticate`.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MeDeleteCredentialRequest {
    /// Step-up token minted by `/me/reauthenticate`.
    #[validate(length(min = 1))]
    pub step_up_token: String,
}

#[utoipa::path(
    get,
    path = "/realms/{realm_name}/me/credentials",
    tag = "auth",
    summary = "List the signed-in user's credentials",
    description = "Returns the credentials (otp, passkey, recovery codes) of the authenticated user. Bearer-only self-service endpoint; no realm admin permission required.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Credentials retrieved successfully", body = MeCredentialsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn me_credentials(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Extension(auth_realm): Extension<AuthenticatedRealm>,
) -> Result<Response<MeCredentialsResponse>, ApiError> {
    // Compare realm *ids* (the token's realm is authoritative, carried by the
    // resolved identity) rather than names parsed from the issuer claim.
    let path_realm_id = state
        .service
        .realm_id_for_name(&realm_name)
        .await
        .map_err(ApiError::from)?;
    if path_realm_id != auth_realm.realm_id {
        return Err(ApiError::Forbidden(
            "token realm does not match the requested realm".into(),
        ));
    }

    let credentials = state
        .service
        .list_credentials_self_service(identity)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(MeCredentialsResponse { data: credentials }))
}

#[utoipa::path(
    delete,
    path = "/realms/{realm_name}/me/credentials/{credential_id}",
    tag = "auth",
    summary = "Delete one of the signed-in user's credentials",
    description = "Deletes a credential owned by the authenticated user. Bearer-only self-service endpoint; the credential must belong to the caller.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("credential_id" = Uuid, Path, description = "Credential ID"),
    ),
    request_body = MeDeleteCredentialRequest,
    responses(
        (status = 200, description = "Credential deleted successfully", body = MeDeleteCredentialResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Credential does not belong to the user", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn me_delete_credential(
    Path((realm_name, credential_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Extension(auth_realm): Extension<AuthenticatedRealm>,
    ValidateJson(payload): ValidateJson<MeDeleteCredentialRequest>,
) -> Result<Response<MeDeleteCredentialResponse>, ApiError> {
    // Compare realm *ids* (the token's realm is authoritative, carried by the
    // resolved identity) rather than names parsed from the issuer claim.
    let path_realm_id = state
        .service
        .realm_id_for_name(&realm_name)
        .await
        .map_err(ApiError::from)?;
    if path_realm_id != auth_realm.realm_id {
        return Err(ApiError::Forbidden(
            "token realm does not match the requested realm".into(),
        ));
    }

    state
        .service
        .delete_credential_self_service(identity, credential_id, payload.step_up_token)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(MeDeleteCredentialResponse {
        message: format!("Credential {credential_id} deleted successfully"),
    }))
}

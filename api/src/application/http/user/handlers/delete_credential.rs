use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::credential::ports::CredentialService;
use ferriskey_core::domain::{
    authentication::value_objects::Identity, credential::entities::DeleteCredentialInput,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteUserCredentialResponse {
    pub message: String,
    pub realm_name: String,
    pub user_id: Uuid,
}

#[utoipa::path(
    delete,
    path = "/{user_id}/credentials/{credential_id}",
    tag = "user",
    summary = "Delete a user credential in a realm",
    description = "Deletes a specific credential for a user in a realm. This action is irreversible and will remove all associated data.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
        ("credential_id" = Uuid, Path, description = "Credential ID"),
    ),
    responses(
        (status = 200, body = DeleteUserCredentialResponse)
    )
)]
pub async fn delete_user_credential(
    Path(realm_name): Path<String>,
    Path(user_id): Path<Uuid>,
    Path(credential_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteUserCredentialResponse>, ApiError> {
    state
        .service
        .delete_credential(
            identity,
            DeleteCredentialInput {
                credential_id,
                realm_name: realm_name.clone(),
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeleteUserCredentialResponse {
        message: format!(
            "Credential with ID {credential_id} for user {user_id} in realm {realm_name} deleted successfully"
        ),
        realm_name,
        user_id,
    }))
}

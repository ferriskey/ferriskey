use axum::extract::State;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    application::http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
    domain::credential::ports::credential_service::CredentialService,
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}/credentials/{credential_id}")]
pub struct DeleteUserCredentialRoute {
    pub realm_name: String,
    pub user_id: Uuid,
    pub credential_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct DeleteUserCredentialResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/{user_id}/credentials/{credential_id}",
    tag = "user",
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
    DeleteUserCredentialRoute {
        realm_name,
        user_id,
        credential_id,
    }: DeleteUserCredentialRoute,
    State(state): State<AppState>,
) -> Result<Response<DeleteUserCredentialResponse>, ApiError> {
    state
        .credential_service
        .delete_by_id(credential_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeleteUserCredentialResponse {
        message: format!(
            "Credential with ID {credential_id} for user {user_id} in realm {realm_name} deleted successfully"
        ),
    }))
}

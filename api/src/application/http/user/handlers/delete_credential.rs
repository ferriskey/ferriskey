use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::user::use_cases::delete_credential_use_case::DeleteCredentialUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}/credentials/{credential_id}")]
pub struct DeleteUserCredentialRoute {
    pub realm_name: String,
    pub user_id: Uuid,
    pub credential_id: Uuid,
}

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
    DeleteUserCredentialRoute {
        realm_name,
        user_id,
        credential_id,
    }: DeleteUserCredentialRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteUserCredentialResponse>, ApiError> {
    state
        .use_case_bundle
        .delete_credential_use_case
        .execute(
            identity,
            DeleteCredentialUseCaseParams {
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

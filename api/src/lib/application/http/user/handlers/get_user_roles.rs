use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    application::{
        auth::Identity,
        http::server::{
            api_entities::{api_error::ApiError, response::Response},
            app_state::AppState,
        },
    },
    domain::role::entities::models::Role,
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}/roles")]
pub struct GetUserRolesRoute {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct GetUserRolesResponse {
    pub data: Vec<Role>,
}

#[utoipa::path(
    get,
    summary = "Get all roles for a specific user",
    path = "",
    tag = "user",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, body = GetUserRolesResponse),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user_roles(
    GetUserRolesRoute { realm_name: _, user_id }: GetUserRolesRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetUserRolesResponse>, ApiError> {
    // Get the user from identity to verify permissions
    let requesting_user = match identity {
        Identity::User(user) => user,
        Identity::Client(client) => {
            state
                .user_service
                .get_by_client_id(client.id)
                .await
                .map_err(|_| ApiError::Forbidden("Client not found".to_string()))?
        }
    };

    // Get the target user's roles
    let roles = state
        .user_service
        .get_user_roles(user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetUserRolesResponse { data: roles }))
} 
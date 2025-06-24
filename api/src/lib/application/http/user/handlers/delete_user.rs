use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    application::{
        auth::Identity,
        http::{
            server::{
                api_entities::{api_error::ApiError, response::Response},
                app_state::AppState,
            },
            user::policies::user_policies::UserPolicy,
        },
    },
    domain::{realm::ports::realm_service::RealmService, user::ports::user_service::UserService},
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}")]
pub struct DeleteUserRoute {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct DeleteUserResponse {
    pub count: u32,
}

#[utoipa::path(
    delete,
    path = "/{user_id}",
    tag = "user",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = String, Path, description = "User ID"),
    ),
)]
pub async fn delete_user(
    DeleteUserRoute {
        realm_name,
        user_id,
    }: DeleteUserRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteUserResponse>, ApiError> {
    let realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let hasPermission = UserPolicy::delete(identity, state.clone(), realm.clone()).await?;

    if !hasPermission {
        return Err(ApiError::Forbidden(
            "You do not have permission to delete users".to_string(),
        ));
    }

    let count = state
        .user_service
        .delete_user(user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeleteUserResponse {
        count: count as u32,
    }))
}

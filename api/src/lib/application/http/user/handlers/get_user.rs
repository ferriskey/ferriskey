use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::domain::realm::ports::RealmService;
use ferriskey_core::domain::user::entities::User;
use ferriskey_core::domain::user::ports::UserService;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}")]
pub struct GetUserRoute {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct UserResponse {
    pub data: User,
}

#[utoipa::path(
    get,
    path = "/{user_id}",
    tag = "user",
    params(
        ("realm_name" = String, Path, description = "Realm name"),  
        ("user_id" = String, Path, description = "User ID"),
    ),
)]
pub async fn get_user(
    GetUserRoute {
        realm_name,
        user_id,
    }: GetUserRoute,
    State(state): State<AppState>,
) -> Result<Response<UserResponse>, ApiError> {
    let _ = state
        .service_bundle
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let user = state
        .service_bundle
        .user_service
        .get_by_id(user_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UserResponse { data: user }))
}

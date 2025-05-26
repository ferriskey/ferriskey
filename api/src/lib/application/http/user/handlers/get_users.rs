use axum::extract::State;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

use crate::{
    application::http::server::{
            api_entities::{api_error::ApiError, response::Response},
            app_state::AppState,
        },
    domain::{
        realm::ports::realm_service::RealmService,
        user::{entities::model::User, ports::user_service::UserService},
    },
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users")]
pub struct GetUsersRoute {
    pub realm_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct UsersResponse {
    pub data: Vec<User>,
}

#[utoipa::path(
    get,
    path = "/{realm_name}/users",
    tag = "user",
    params(
        ("realm_name" = String, Path, description = "Realm name"),  
    ),
)]
pub async fn get_users(
    GetUsersRoute { realm_name }: GetUsersRoute,
    State(state): State<AppState>,
) -> Result<Response<UsersResponse>, ApiError> {
    let realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let users = state
        .user_service
        .find_by_realm_id(realm.id)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UsersResponse { data: users }))
}

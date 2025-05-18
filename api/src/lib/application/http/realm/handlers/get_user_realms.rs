use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

use crate::{
    application::{
        auth::Identity,
        http::server::{
            api_entities::{api_error::ApiError, response::Response},
            app_state::AppState,
        },
    },
    domain::{realm::entities::realm::Realm, user::ports::user_service::UserService},
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/@me/realms")]
pub struct GetUserRealmsRoute {
    pub realm_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct UserRealmsResponse {
    pub data: Vec<Realm>,
}

#[utoipa::path(
    get,
    summary = "Get user realms",
    path = "/users/@me/realms",
    tag = "realm",
    security(
        ("Authorization" = ["Bearer"]),
    ),
    responses(
        (status = 200, body = UserRealmsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn get_user_realms(
    GetUserRealmsRoute { realm_name }: GetUserRealmsRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UserRealmsResponse>, ApiError> {
    let user = match identity {
        Identity::User(user) => user,
        Identity::Client(client) => {
            let service_account = state
                .user_service
                .get_by_client_id(client.id)
                .await
                .map_err(|_| ApiError::Forbidden("Client not found".to_string()))?;

            service_account
        }
    };
    let realms = state
        .user_service
        .get_user_realms(user, realm_name)
        .await
        .map_err(|_| ApiError::Forbidden("User not found".to_string()))?;

    Ok(Response::OK(UserRealmsResponse { data: realms }))
}

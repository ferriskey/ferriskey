use crate::application::http::server::api_entities::{api_error::ApiError, response::Response};
use crate::application::http::server::app_state::AppState;
use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::realm::entities::Realm;
use ferriskey_core::domain::user::ports::UserService;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

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
    _: GetUserRealmsRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UserRealmsResponse>, ApiError> {
    let user = match identity {
        Identity::User(user) => user,
        Identity::Client(client) => state
            .service_bundle
            .user_service
            .get_by_client_id(client.id)
            .await
            .map_err(|_| ApiError::Forbidden("Client not found".to_string()))?,
    };

    let realm = user.realm.clone().ok_or(ApiError::Forbidden(
        "User does not belong to any realm".to_string(),
    ))?;

    let realms = state
        .service_bundle
        .user_service
        .get_user_realms(user, realm.name)
        .await
        .map_err(|_| ApiError::Forbidden("User not found".to_string()))?;

    Ok(Response::OK(UserRealmsResponse { data: realms }))
}

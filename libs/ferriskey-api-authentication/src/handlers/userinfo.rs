use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::{
    ports::AuthService,
    value_objects::{GetUserInfoInput, Identity, UserInfoResponse},
};

use ferriskey_api_core::api_entities::{api_error::ApiError, response::Response};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::decoded_token::OptionalToken;

#[utoipa::path(
    get,
    path = "/protocol/openid-connect/userinfo",
    tag = "auth",
    summary = "Get user info",
    description = "Retrieves the user information for the authenticated user in a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = UserInfoResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn get_userinfo(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    OptionalToken(jwt): OptionalToken,
) -> Result<Response<UserInfoResponse>, ApiError> {
    let jwt = jwt.ok_or(ApiError::Unauthorized("not authorized".into()))?;
    let user_info = state
        .service
        .get_userinfo(
            identity,
            GetUserInfoInput {
                realm_name,
                token: jwt.token,
                claims: jwt.claims,
            },
        )
        .await?;

    Ok(Response::OK(user_info))
}

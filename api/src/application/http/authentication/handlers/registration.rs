use axum::extract::{Path, State};
use ferriskey_core::domain::{
    authentication::{entities::JwtToken, ports::AuthService, value_objects::RegisterUserInput},
    realm::ports::RealmService,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::application::{
    http::server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
    url::FullUrl,
};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct RegistrationRequest {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,

    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/registrations",
    tag = "auth",
    summary = "Register a new user",
    description = "Register a new user in the specified realm",
    request_body = RegistrationRequest,
    responses(
        (status = 201, body = JwtToken),
        (status = 403, description = "User registration is disabled for this realm")
    ),
    params(
        ("realm_name" = String, Path, description = "The realm name" )
    ),
)]
pub async fn registration_handler(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, url): FullUrl,
    ValidateJson(req): ValidateJson<RegistrationRequest>,
) -> Result<Response<JwtToken>, ApiError> {
    let settings = state.service.get_login_settings(realm_name.clone()).await?;

    if !settings.user_registration_enabled {
        return Err(ApiError::Forbidden("registration disabled".to_string()));
    }

    let jwt_token = state
        .service
        .register_user(
            url,
            RegisterUserInput {
                email: req.email,
                first_name: req.first_name,
                last_name: req.last_name,
                password: req.password,
                realm_name,
                username: req.username,
            },
        )
        .await?;

    Ok(Response::Created(jwt_token))
}

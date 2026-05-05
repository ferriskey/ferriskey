use axum::extract::{Path, State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::{
    authentication::{
        ports::AuthService,
        value_objects::{RegisterUserInput, RegisterUserOutput},
    },
    realm::ports::RealmService,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::auth::root_scoped_base_url;
use crate::application::{
    http::server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
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
        (status = 201, body = RegisterUserOutput),
        (status = 400, description = "Email already exists", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "User registration is disabled for this realm", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "The realm name" )
    ),
)]
pub async fn registration_handler(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, url): FullUrl,
    cookie: CookieManager,
    ValidateJson(req): ValidateJson<RegistrationRequest>,
) -> Result<Response<RegisterUserOutput>, ApiError> {
    let settings = state.service.get_login_settings(realm_name.clone()).await?;

    if !settings.user_registration_enabled {
        return Err(ApiError::Forbidden("registration disabled".into()));
    }

    let session_code = cookie
        .get("FERRISKEY_SESSION")
        .and_then(|c| Uuid::parse_str(c.value()).ok());

    let url = root_scoped_base_url(&url, &state.args.server.root_path);
    let output = state
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
                session_code,
            },
        )
        .await?;

    Ok(Response::Created(output))
}

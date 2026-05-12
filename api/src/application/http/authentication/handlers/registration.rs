use axum::extract::{Path, State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::{
    authentication::{
        entities::JwtToken,
        ports::AuthService,
        value_objects::{RegisterUserInput, RegisterUserOutput, RegisterUserUrlContext},
    },
    realm::ports::RealmService,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::auth::root_scoped_base_url;
use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use crate::application::url::FullUrl;

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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PendingVerificationResponse {
    pub message: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RedirectRegistrationResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum RegistrationResponse {
    Authenticated(JwtToken),
    Redirect(RedirectRegistrationResponse),
    PendingVerification(PendingVerificationResponse),
}

fn registration_verification_base_url(webapp_url: &str) -> String {
    webapp_url.trim_end_matches('/').to_string()
}

fn registration_url_context(
    webapp_url: &str,
    request_base_url: &str,
    root_path: &str,
) -> RegisterUserUrlContext {
    RegisterUserUrlContext {
        issuer_base_url: root_scoped_base_url(request_base_url, root_path),
        verification_base_url: registration_verification_base_url(webapp_url),
    }
}

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/registrations",
    tag = "auth",
    summary = "Register a new user",
    description = "Register a new user in the specified realm. Returns JWT tokens if email verification is disabled, or a pending verification message if enabled.",
    request_body = RegistrationRequest,
    responses(
        (status = 201, body = RegistrationResponse),
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
    FullUrl(_, base_url): FullUrl,
    cookie: CookieManager,
    ValidateJson(req): ValidateJson<RegistrationRequest>,
) -> Result<Response<RegistrationResponse>, ApiError> {
    let settings = state.service.get_login_settings(realm_name.clone()).await?;

    if !settings.user_registration_enabled {
        return Err(ApiError::Forbidden("registration disabled".into()));
    }

    let session_code = cookie
        .get("FERRISKEY_SESSION")
        .and_then(|c| Uuid::parse_str(c.value()).ok());

    let url_context = registration_url_context(
        &state.args.webapp_url,
        &base_url,
        &state.args.server.root_path,
    );
    let output = state
        .service
        .register_user(
            url_context,
            RegisterUserInput {
                email: req.email,
                first_name: req.first_name,
                last_name: req.last_name,
                password: req.password,
                realm_name: realm_name.clone(),
                username: req.username,
                session_code,
            },
        )
        .await?;

    match output {
        RegisterUserOutput::Authenticated(token) => Ok(Response::Created(
            RegistrationResponse::Authenticated(token),
        )),
        RegisterUserOutput::Redirect { url } => Ok(Response::Created(
            RegistrationResponse::Redirect(RedirectRegistrationResponse { url }),
        )),
        RegisterUserOutput::PendingVerification { message, user_id } => Ok(Response::Created(
            RegistrationResponse::PendingVerification(PendingVerificationResponse {
                message,
                user_id,
            }),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json;

    use super::*;

    #[test]
    fn test_registration_request_deserialization() {
        let json = r#"{
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123",
            "first_name": "John",
            "last_name": "Doe"
        }"#;

        let request: RegistrationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "testuser");
        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.password, "password123");
        assert_eq!(request.first_name, Some("John".to_string()));
        assert_eq!(request.last_name, Some("Doe".to_string()));
    }

    #[test]
    fn test_registration_request_deserialization_with_defaults() {
        let json = r#"{}"#;

        let request: RegistrationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "");
        assert_eq!(request.email, "");
        assert_eq!(request.password, "");
        assert_eq!(request.first_name, None);
        assert_eq!(request.last_name, None);
    }

    #[test]
    fn test_pending_verification_response_serialization() {
        let user_id = Uuid::new_v4();
        let response = PendingVerificationResponse {
            message: "Please check your email".to_string(),
            user_id,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Please check your email"));
        assert!(json.contains(&user_id.to_string()));
    }

    #[test]
    fn test_registration_response_pending_verification_serialization() {
        let user_id = Uuid::new_v4();
        let response = RegistrationResponse::PendingVerification(PendingVerificationResponse {
            message: "Please verify your email".to_string(),
            user_id,
        });

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "status": "pending_verification",
                "data": {
                    "message": "Please verify your email",
                    "user_id": user_id,
                }
            })
        );
    }

    #[test]
    fn test_registration_response_redirect_serialization() {
        let response = RegistrationResponse::Redirect(RedirectRegistrationResponse {
            url: "https://client.example/callback".to_string(),
        });

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "status": "redirect",
                "data": {
                    "url": "https://client.example/callback",
                }
            })
        );
    }

    #[test]
    fn test_registration_response_authenticated_serialization() {
        let response = RegistrationResponse::Authenticated(JwtToken::new(
            "access-token".to_string(),
            "Bearer".to_string(),
            "refresh-token".to_string(),
            300,
            600,
            None,
            None,
        ));

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "status": "authenticated",
                "data": {
                    "access_token": "access-token",
                    "token_type": "Bearer",
                    "refresh_token": "refresh-token",
                    "expires_in": 300,
                    "refresh_expires_in": 600,
                }
            })
        );
    }

    #[test]
    fn registration_verification_base_url_uses_webapp_url() {
        let base_url = registration_verification_base_url("http://localhost:3000/");

        assert_eq!(base_url, "http://localhost:3000");
    }

    #[test]
    fn registration_url_context_separates_issuer_and_verification_bases() {
        let context = registration_url_context(
            "https://account.longcipher.com/",
            "https://ferriskey-api.longcipher.com",
            "/auth",
        );

        assert_eq!(
            context.issuer_base_url,
            "https://ferriskey-api.longcipher.com/auth"
        );
        assert_eq!(
            context.verification_base_url,
            "https://account.longcipher.com"
        );
    }
}

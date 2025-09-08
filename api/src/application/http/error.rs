use ferriskey_core::domain::common::entities::app_errors::CoreError;

use crate::application::http::server::api_entities::api_error::ApiError;

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::NotFound => Self::NotFound("Resource not found".to_string()),
            CoreError::AlreadyExists => Self::BadRequest("Resource already exists".to_string()),
            CoreError::Invalid => Self::BadRequest("Invalid resource".to_string()),
            CoreError::Forbidden(msg) => Self::Forbidden(msg),
            CoreError::InternalServerError => {
                Self::InternalServerError("Internal server error".to_string())
            }
            CoreError::RedirectUriNotFound => Self::NotFound("Redirect URI not found".to_string()),
            CoreError::InvalidRedirectUri => Self::BadRequest("Invalid redirect URI".to_string()),
            CoreError::InvalidClient => Self::Unauthorized("Invalid client".to_string()),
            CoreError::InvalidRealm => Self::Unauthorized("Invalid realm".to_string()),
            CoreError::InvalidUser => Self::Unauthorized("Invalid user".to_string()),
            CoreError::InvalidPassword => Self::Unauthorized("Invalid password".to_string()),
            CoreError::InvalidState => Self::BadRequest("Invalid state".to_string()),
            CoreError::InvalidRefreshToken => {
                Self::Unauthorized("Invalid refresh token".to_string())
            }
            CoreError::InvalidClientSecret => {
                Self::Unauthorized("Invalid client secret".to_string())
            }
            CoreError::InvalidRequest => {
                Self::BadRequest("Invalid authorization request".to_string())
            }
            CoreError::ServiceAccountNotFound => {
                Self::NotFound("Service account not found".to_string())
            }
            CoreError::HashPasswordError(msg) => {
                Self::InternalServerError(format!("Hash password error: {}", msg))
            }
            CoreError::VerifyPasswordError(msg) => {
                Self::InternalServerError(format!("Verify password error: {}", msg))
            }
            CoreError::DeletePasswordCredentialError => {
                Self::InternalServerError("Failed to delete password credential".to_string())
            }
            CoreError::CreateCredentialError => {
                Self::InternalServerError("Failed to create credential".to_string())
            }
            CoreError::GetPasswordCredentialError => {
                Self::InternalServerError("Failed to get password credential".to_string())
            }
            CoreError::GetUserCredentialsError => {
                Self::InternalServerError("Failed to get user credentials".to_string())
            }
            CoreError::DeleteCredentialError => {
                Self::InternalServerError("Failed to delete credential".to_string())
            }
            CoreError::TokenGenerationError(msg) => {
                Self::InternalServerError(format!("Token generation error: {}", msg))
            }
            CoreError::TokenValidationError(msg) => {
                Self::Unauthorized(format!("Token validation error: {}", msg))
            }
            CoreError::TokenParsingError(msg) => {
                Self::BadRequest(format!("Token parsing error: {}", msg))
            }
            CoreError::TokenExpirationError(msg) => {
                Self::Unauthorized(format!("Token expiration error: {}", msg))
            }
            CoreError::RealmKeyNotFound => {
                Self::InternalServerError("Realm key not found".to_string())
            }
            CoreError::InvalidToken => Self::Unauthorized("Invalid token".to_string()),
            CoreError::ExpiredToken => Self::Unauthorized("Expired token".to_string()),
            CoreError::InvalidKey(msg) => Self::BadRequest(format!("Invalid key: {}", msg)),
            CoreError::SessionNotFound => Self::NotFound("Session not found".to_string()),
            CoreError::SessionExpired => Self::Unauthorized("Session expired".to_string()),
            CoreError::InvalidSession => Self::Unauthorized("Invalid session".to_string()),
            CoreError::SessionCreateError => {
                Self::InternalServerError("Failed to create session".to_string())
            }
            CoreError::SessionDeleteError => {
                Self::InternalServerError("Failed to delete session".to_string())
            }
            CoreError::InvalidTotpSecretFormat => {
                Self::BadRequest("Invalid TOTP secret format".to_string())
            }
            CoreError::TotpGenerationFailed(msg) => {
                Self::InternalServerError(format!("TOTP generation failed: {}", msg))
            }
            CoreError::TotpVerificationFailed(msg) => {
                Self::Unauthorized(format!("TOTP verification failed: {}", msg))
            }
            CoreError::CannotDeleteMasterRealm => {
                Self::Forbidden("Cannot delete master realm".to_string())
            }
            CoreError::WebhookNotFound => Self::NotFound("Webhook not found".to_string()),
            CoreError::WebhookForbidden => Self::Forbidden("Webhook forbidden".to_string()),
            CoreError::FailedWebhookNotification(msg) => {
                Self::InternalServerError(format!("Failed to notify webhook: {}", msg))
            }
            CoreError::WebhookRealmNotFound => {
                Self::NotFound("Realm not found for webhook".to_string())
            }
            CoreError::CreateClientError => {
                Self::InternalServerError("Failed to create client".to_string())
            }
            CoreError::ServiceUnavailable(msg) => Self::ServiceUnavailable(msg),
        }
    }
}

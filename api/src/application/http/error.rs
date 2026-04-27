use ferriskey_core::domain::common::entities::app_errors::CoreError;
use ferriskey_core::domain::user::entities::RequiredAction;

use crate::application::http::server::api_entities::api_error::ApiError;

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::NotFound => Self::NotFound("Resource not found".into()),
            CoreError::AlreadyExists => Self::BadRequest("Resource already exists".into()),
            CoreError::EmailAlreadyExists => {
                Self::BadRequest("Email already exists in this realm".into())
            }
            CoreError::Invalid => Self::BadRequest("Invalid resource".into()),
            CoreError::InvalidRequiredAction(action) => {
                let allowed = RequiredAction::allowed_values().join(", ");
                Self::BadRequest(format!(
                    "Invalid required action: {}. Allowed values: {}",
                    action, allowed
                ).into())
            }
            CoreError::Forbidden(msg) => Self::Forbidden(msg.into()),
            CoreError::InternalServerError => {
                        Self::InternalServerError("Internal server error".into())
                    }
            CoreError::RedirectUriNotFound => Self::NotFound("Redirect URI not found".into()),
            CoreError::InvalidRedirectUri => Self::BadRequest("Invalid redirect URI".into()),
            CoreError::InvalidClient => Self::Unauthorized("Invalid client".into()),
            CoreError::InvalidRealm => Self::Unauthorized("Invalid realm".into()),
            CoreError::InvalidUser => Self::Unauthorized("Invalid user".into()),
            CoreError::InvalidPassword => Self::Unauthorized("Invalid password".into()),
            CoreError::InvalidState => Self::BadRequest("Invalid state".into()),
            CoreError::InvalidRefreshToken => {
                        Self::Unauthorized("Invalid refresh token".into())
                    }
            CoreError::InvalidClientSecret => {
                        Self::Unauthorized("Invalid client secret".into())
                    }
            CoreError::InvalidRequest => {
                        Self::BadRequest("Invalid authorization request".into())
                    }
            CoreError::ServiceAccountNotFound => {
                        Self::NotFound("Service account not found".into())
                    }
            CoreError::HashPasswordError(msg) => {
                        Self::InternalServerError(format!("Hash password error: {}", msg).into())
                    }
            CoreError::VerifyPasswordError(msg) => {
                        Self::InternalServerError(format!("Verify password error: {}", msg).into())
                    }
            CoreError::DeletePasswordCredentialError => {
                        Self::InternalServerError("Failed to delete password credential".into())
                    }
            CoreError::CreateCredentialError => {
                        Self::InternalServerError("Failed to create credential".into())
                    }
            CoreError::GetPasswordCredentialError => {
                        Self::InternalServerError("Failed to get password credential".into())
                    }
            CoreError::GetUserCredentialsError => {
                        Self::InternalServerError("Failed to get user credentials".into())
                    }
            CoreError::DeleteCredentialError => {
                        Self::InternalServerError("Failed to delete credential".into())
                    }
            CoreError::TokenGenerationError(msg) => {
                        Self::InternalServerError(format!("Token generation error: {}", msg).into())
                    }
            CoreError::TokenValidationError(msg) => {
                        Self::Unauthorized(format!("Token validation error: {}", msg).into())
                    }
            CoreError::TokenParsingError(msg) => {
                        Self::BadRequest(format!("Token parsing error: {}", msg).into())
                    }
            CoreError::TokenExpirationError(msg) => {
                        Self::Unauthorized(format!("Token expiration error: {}", msg).into())
                    }
            CoreError::RealmKeyNotFound => {
                        Self::InternalServerError("Realm key not found".into())
                    }
            CoreError::InvalidToken => Self::Unauthorized("Invalid token".into()),
            CoreError::ExpiredToken => Self::Unauthorized("Expired token".into()),
            CoreError::InvalidKey(msg) => Self::BadRequest(format!("Invalid key: {}", msg).into()),
            CoreError::SessionNotFound => Self::NotFound("Session not found".into()),
            CoreError::SessionExpired => Self::Unauthorized("Session expired".into()),
            CoreError::InvalidSession => Self::Unauthorized("Invalid session".into()),
            CoreError::SessionCreateError => {
                        Self::InternalServerError("Failed to create session".into())
                    }
            CoreError::SessionDeleteError => {
                        Self::InternalServerError("Failed to delete session".into())
                    }
            CoreError::InvalidTotpSecretFormat => {
                        Self::BadRequest("Invalid TOTP secret format".into())
                    }
            CoreError::TotpGenerationFailed(msg) => {
                        Self::InternalServerError(format!("TOTP generation failed: {}", msg).into())
                    }
            CoreError::TotpVerificationFailed(msg) => {
                        Self::Unauthorized(format!("TOTP verification failed: {}", msg).into())
                    }
            CoreError::CannotDeleteMasterRealm => {
                        Self::Forbidden("Cannot delete master realm".into())
                    }
            CoreError::WebhookNotFound => Self::NotFound("Webhook not found".into()),
            CoreError::WebhookForbidden => Self::Forbidden("Webhook forbidden".into()),
            CoreError::FailedWebhookNotification(msg) => {
                        Self::InternalServerError(format!("Failed to notify webhook: {}", msg).into())
                    }
            CoreError::WebhookRealmNotFound => {
                        Self::NotFound("Realm not found for webhook".into())
                    }
            CoreError::CreateClientError => {
                        Self::InternalServerError("Failed to create client".into())
                    }
            CoreError::ServiceUnavailable(msg) => Self::ServiceUnavailable(msg.into()),
            CoreError::RecoveryCodeGenError(msg) => Self::BadRequest(msg.into()),
            CoreError::RecoveryCodeBurnError(msg) => Self::BadRequest(msg.into()),
            CoreError::AuthorizationCodeStorageFailed => {
                Self::InternalServerError("".into())
            },
            CoreError::AuthSessionExpectedState => {
                Self::InternalServerError("".into())
            },
            CoreError::WebAuthnMissingChallenge => {
                Self::BadRequest("There is no current webauthn challenge for this session. Make sure you request one from the server before attempting an authentication.".into())
            },
            CoreError::WebAuthnCredentialNotFound => {
                Self::BadRequest("Missing webauthn credential for the provided id. Have you created a webauthn credential first ?".into())
            }
            CoreError::WebAuthnChallengeFailed => {
                Self::Unauthorized("Webauthn challenged failed. A new one must be requested to retry.".into())
            }
            CoreError::MagicLinkNotEnabled => {
                Self::BadRequest("Magic link authentication is not enabled for this realm".into())
            }
            CoreError::InvalidMagicLink => {
                Self::Unauthorized("Invalid magic link token".into())
            }
            CoreError::MagicLinkExpired => {
                Self::Unauthorized("Magic link has expired".into())
            }
            CoreError::MagicLinkAlreadyUsed => {
                Self::BadRequest("Magic link has already been used".into())
            }
            CoreError::ProviderNotFound => {
                Self::NotFound("Provider not found".into())
            }
            CoreError::ProviderNameAlreadyExists => {
                Self::BadRequest("Provider name already exists".into())
            }
            CoreError::InvalidProviderConfiguration(msg) => {
                Self::BadRequest(format!("Invalid provider configuration: {}", msg).into())
            }
            CoreError::ProviderDisabled => {
                Self::Forbidden("Provider is disabled".into())
            }
            CoreError::InvalidProviderUrl => {
                Self::BadRequest("Invalid provider URL".into())
            },
            CoreError::External(msg) => Self::ServiceUnavailable(format!("External service error: {}", msg).into()),
            CoreError::Database(msg) => Self::InternalServerError(format!("Database error: {}", msg).into()),
            CoreError::Configuration(msg) => Self::InternalServerError(format!("Configuration error: {}", msg).into()),
            CoreError::FederationAuthenticationFailed(msg) => Self::Unauthorized(format!("Federation authentication error: {}", msg).into()),

            // Broker (SSO) errors
            CoreError::BrokerSessionNotFound => {
                Self::BadRequest("Invalid or expired SSO session".into())
            }
            CoreError::BrokerSessionExpired => {
                Self::BadRequest("SSO session expired, please try again".into())
            }
            CoreError::InvalidBrokerState => {
                Self::BadRequest("Invalid state parameter".into())
            }
            CoreError::IdpTokenExchangeFailed(msg) => {
                Self::ServiceUnavailable(format!("Identity provider error: {}", msg).into())
            }
            CoreError::IdpUserInfoFailed(msg) => {
                Self::ServiceUnavailable(format!("Failed to retrieve user info: {}", msg).into())
            }
            CoreError::IdpAuthenticationFailed(msg) => {
                Self::Unauthorized(format!("Identity provider authentication failed: {}", msg).into())
            }
            CoreError::UserLinkingFailed(msg) => {
                Self::InternalServerError(format!("User linking failed: {}", msg).into())
            }
            CoreError::LinkOnlyUserNotFound => {
                Self::Forbidden("No existing account found for linking".into())
            }
            CoreError::LinkNotFound => {
                Self::NotFound("Identity provider link not found".into())
            }
            CoreError::InvalidIdToken => {
                Self::BadRequest("Invalid ID token from identity provider".into())
            }
            CoreError::MissingAuthorizationCode => {
                Self::BadRequest("Missing authorization code from identity provider".into())
            }
            CoreError::UserNotFound => {
                Self::NotFound("User not found".into())
            }
            CoreError::ClientNotFound => {
                Self::NotFound("Client not found".into())
            }
            CoreError::HintsNotFound => {
                Self::NotFound("Account hints not found".into())
            }
            CoreError::InvalidScope(description) => Self::OAuthError {
                error: "invalid_scope".into(),
                error_description: description.into(),
            },
            CoreError::UserDisabled => Self::Forbidden("User account is disabled".into()),
            CoreError::ClientUnderMaintenance(reason) => Self::ServiceUnavailable(reason.into()),
            CoreError::EmailTemplateNotFound => {
                Self::NotFound("Email template not found".into())
            }
            CoreError::NoActiveEmailTemplate(email_type) => {
                Self::NotFound(format!("No active email template for type: {email_type}").into())
            }
            CoreError::InvalidEmailTemplateStructure(msg) => {
                Self::BadRequest(format!("Invalid email template structure: {msg}").into())
            }
            CoreError::EmailTemplateRenderError(msg) => {
                Self::InternalServerError(format!("Email template render error: {msg}").into())
            }
            CoreError::InvalidOrExpiredToken => {
                Self::BadRequest("Invalid or expired email verification token".into())
            }
            CoreError::EmailVerificationTemplateNotConfigured => {
                Self::BadRequest("Email verification template is not configured for this realm".into())
            }
        }
    }
}

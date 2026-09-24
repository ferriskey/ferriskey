use ferriskey_core::domain::{
    authentication::device_flow::error::DeviceFlowError, common::entities::app_errors::CoreError,
    credential::entities::CredentialError, password_policy::error::PasswordPolicyViolation,
    portal_theme::validation::MissingBlocks, user::entities::RequiredAction,
};
use serde_json::{from_str, to_string};

use crate::api_entities::api_error::{ApiError, ApiErrorBody, ValidationError};

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        let reason = error.reason();

        let api_error = match error {
            CoreError::NotFound => Self::NotFound("Resource not found".into()),
            CoreError::AlreadyExists => Self::BadRequest("Resource already exists".into()),
            CoreError::EmailAlreadyExists => {
                Self::BadRequest("Email already exists in this realm".into())
            }
            CoreError::InvalidCredentials => Self::Unauthorized("Invalid credentials".into()),
            CoreError::UsernameAlreadyExists => {
                Self::BadRequest("Username already exists in this realm".into())
            }
            CoreError::UserIdAlreadyExists => {
                Self::Conflict("A user already exists with this id".into())
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
            CoreError::ElevationRequired => Self::Forbidden(ApiErrorBody::new(
                "This operation requires a fresh re-authentication",
                reason,
            )),
            CoreError::PrimaryProofRequired => Self::Forbidden(ApiErrorBody::new(
                "This operation requires re-authenticating with a primary credential",
                reason,
            )),
            CoreError::NoLocalPassword => Self::Conflict(ApiErrorBody::new(
                "This account signs in through an external directory and has no local password",
                reason,
            )),
            CoreError::LastSignInMeans => Self::Conflict(ApiErrorBody::new(
                "Removing this credential would leave the account with no way to sign in",
                reason,
            )),
            CoreError::MfaFactorRequired => Self::Conflict(ApiErrorBody::new(
                "Removing this credential would leave the account without the second factor this realm requires",
                reason,
            )),
            CoreError::InternalServerError => {
                        Self::InternalServerError("Internal server error".into())
                    }
            CoreError::RedirectUriNotFound => {
                Self::NotFound("No redirect URI is registered for this client".into())
            }
            CoreError::InvalidRedirectUri => {
                Self::BadRequest("Redirect URI is not allowed for this client".into())
            }
            CoreError::WebOriginNotFound => {
                Self::NotFound("No web origin is registered under this identifier".into())
            }
            CoreError::InvalidWebOrigin(detail) => {
                Self::BadRequest(CoreError::InvalidWebOrigin(detail).to_string().into())
            }
            CoreError::SamlConfigNotFound => {
                Self::NotFound("No SAML configuration is registered for this client".into())
            }
            CoreError::InvalidSamlConfig(detail) => {
                Self::BadRequest(CoreError::InvalidSamlConfig(detail).to_string().into())
            }
            CoreError::SamlAttributeMapperNotFound => Self::NotFound(
                "No SAML attribute mapper is registered under this identifier".into(),
            ),
            CoreError::InvalidSamlAttributeMapper(detail) => Self::BadRequest(
                CoreError::InvalidSamlAttributeMapper(detail).to_string().into(),
            ),
            CoreError::InvalidClient => Self::Unauthorized("Invalid client".into()),
            CoreError::InvalidRealm => {
                Self::NotFound(ApiErrorBody::new("Resource not found", "not_found"))
            }
            CoreError::RealmAlreadyExists(name) => {
                Self::Conflict(CoreError::RealmAlreadyExists(name).to_string().into())
            }
            CoreError::InvalidUser => Self::Unauthorized("Invalid user".into()),
            CoreError::InvalidPassword => Self::Unauthorized("Invalid password".into()),
            CoreError::InvalidState => Self::BadRequest("Invalid state".into()),
            CoreError::InvalidRefreshToken => {
                        Self::Unauthorized("Invalid refresh token".into())
                    }
            // Presenting a token whose session was revoked to a protected
            // resource is an authentication failure. The token endpoint maps
            // this to `invalid_grant` itself (RFC 6749 §5.2) before it gets here.
            CoreError::SessionRevoked => {
                        Self::Unauthorized("Session revoked".into())
                    }
            CoreError::InvalidGrant(description) => Self::OAuthError {
                error: "invalid_grant".into(),
                error_description: description.into(),
            },
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
            CoreError::InvalidWebhookEndpoint(message) => Self::BadRequest(message.into()),
            CoreError::InvalidWebhookRetryPolicy(message) => Self::BadRequest(message.into()),
            CoreError::WebhookDeliveryNotFound => {
                        Self::NotFound("Webhook delivery not found".into())
                    }
            CoreError::WebhookDeliveryNotReplayable => Self::Conflict(
                        "Webhook delivery is still in flight and cannot be replayed".into(),
                    ),
            CoreError::CreateClientError => {
                        Self::InternalServerError("Failed to create client".into())
                    }
            CoreError::ClientIdAlreadyExists(client_id) => {
                        Self::Conflict(CoreError::ClientIdAlreadyExists(client_id).to_string().into())
                    }
            CoreError::ServiceUnavailable(msg) => Self::ServiceUnavailable(msg.into()),
            CoreError::RecoveryCodeGenError(msg) => Self::BadRequest(msg.into()),
            CoreError::RecoveryCodeBurnError(msg) => Self::BadRequest(msg.into()),
            CoreError::AuthorizationCodeStorageFailed => {
                Self::InternalServerError("Failed to store the authorization code".into())
            },
            CoreError::ProtocolNotSupported(protocol) => {
                Self::BadRequest(CoreError::ProtocolNotSupported(protocol).to_string().into())
            }
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
            CoreError::AccountLocked => Self::Unauthorized(
                "Account is temporarily locked due to too many failed login attempts".into(),
            ),
            CoreError::ClientUnderMaintenance(detail) => Self::ServiceUnavailable(detail.into()),
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
            CoreError::PortalThemePageInvalid(details) => {
                Self::validation_error("tree", reason, details)
            }
            CoreError::PortalThemeInvalidForActivation(details) => {
                match from_str::<Vec<MissingBlocks>>(&details) {
                    Ok(items) => Self::validation_errors(
                        items
                            .into_iter()
                            .map(|item| ValidationError::new(
                                format!("tree.{:?}", item.page_type),
                                reason,
                                to_string(&item).unwrap_or_default(),
                            ))
                            .collect(),
                    ),
                    Err(_) => Self::validation_error("tree", reason, details),
                }
            }
            CoreError::PortalThemeActive => {
                Self::BadRequest("Portal theme is currently active and cannot be deleted".into())
            }
            CoreError::PortalLayoutDefault => Self::BadRequest(
                "Portal layout is the realm default and cannot be deleted".into(),
            ),
            CoreError::PortalLayoutInUse => Self::BadRequest(
                "Portal layout is referenced by one or more themes and cannot be deleted".into(),
            ),
            CoreError::PortalLayoutInvalidTree(details) => {
                Self::validation_error("tree", reason, details)
            }
            CoreError::PasswordPolicyViolation(details) => {
                match PasswordPolicyViolation::decode(&details) {
                    Some(violations) => Self::validation_errors(
                        violations
                            .into_iter()
                            .map(|violation| {
                                ValidationError::new("password", violation.code, violation.message)
                            })
                            .collect(),
                    ),
                    None => Self::validation_error("password", reason, details),
                }
            }
            CoreError::InvalidPasswordHash(details) => {
                Self::validation_error("secret_data", reason, details)
            }
            CoreError::PasswordCredentialAlreadyExists => {
                Self::Conflict("User already has a password credential".into())
            }
            // PKCE errors (RFC 7636) → OAuth2 invalid_request / invalid_grant
            CoreError::PkceRequired => Self::OAuthError {
                error: "invalid_request".into(),
                error_description: "PKCE is required for this client. Send code_challenge (S256) with the authorization request.".into(),
            },
            CoreError::InvalidAuthorizationCode => Self::OAuthError {
                error: "invalid_grant".into(),
                error_description: "The authorization code is invalid, expired, already used, or was not issued to this client.".into(),
            },
            CoreError::InvalidCodeVerifier => Self::OAuthError {
                error: "invalid_grant".into(),
                error_description: "code_verifier does not match code_challenge".into(),
            },
            CoreError::CodeChallengeMissing => Self::OAuthError {
                error: "invalid_request".into(),
                error_description: "code_challenge is required".into(),
            },
            CoreError::CodeVerifierMissing => Self::OAuthError {
                error: "invalid_grant".into(),
                error_description: "code_verifier is required when code_challenge was used at authorization".into(),
            },
        };

        api_error.with_default_reason(reason)
    }
}

impl From<DeviceFlowError> for ApiError {
    fn from(error: DeviceFlowError) -> Self {
        // Token endpoint errors follow the RFC 6749 §5.2 shape (HTTP 400):
        // `{ "error": "...", "error_description": "..." }`. The device flow
        // polling codes are defined by RFC 8628 §3.5.
        let oauth = |code: &'static str, description: &str| Self::OAuthError {
            error: code.into(),
            error_description: description.to_string().into(),
        };

        match error {
            DeviceFlowError::AuthorizationPending => oauth(
                "authorization_pending",
                "The authorization request is still pending.",
            ),
            DeviceFlowError::SlowDown => oauth("slow_down", "Polling too frequently; slow down."),
            DeviceFlowError::ExpiredToken => oauth("expired_token", "The device code has expired."),
            DeviceFlowError::AccessDenied => {
                oauth("access_denied", "The authorization request was denied.")
            }
            DeviceFlowError::InvalidDeviceCode => {
                oauth("invalid_grant", "The device code is invalid or unknown.")
            }
            DeviceFlowError::InvalidUserCode => {
                oauth("invalid_grant", "The user code is invalid or unknown.")
            }
            DeviceFlowError::InvalidClient => {
                oauth("invalid_client", "Client authentication failed.")
            }
            DeviceFlowError::UnauthorizedClient => oauth(
                "unauthorized_client",
                "The client is not authorized to use the device authorization grant.",
            ),
            DeviceFlowError::InvalidScope => oauth(
                "invalid_scope",
                "The requested scope is not permitted for this client.",
            ),
            DeviceFlowError::Forbidden => Self::Forbidden(ApiErrorBody::new(
                "You cannot act on a device session of another realm",
                "device_session_forbidden",
            )),
            DeviceFlowError::UserCodeGenerationExhausted => {
                Self::InternalServerError(ApiErrorBody::new(
                    "Failed to generate a unique user code",
                    "user_code_generation_exhausted",
                ))
            }
            DeviceFlowError::TokenIssuance(msg) => Self::InternalServerError(ApiErrorBody::new(
                format!("Token issuance failed: {msg}"),
                "token_issuance_failed",
            )),
            DeviceFlowError::Repository(_) => Self::InternalServerError(ApiErrorBody::new(
                "Internal server error",
                "internal_server_error",
            )),
        }
    }
}

impl From<CredentialError> for ApiError {
    fn from(value: CredentialError) -> Self {
        match value {
            CredentialError::CreateCredentialError => ApiError::InternalServerError(
                ApiErrorBody::new("Failed to create credential", "create_credential_error"),
            ),
            CredentialError::GetUserCredentialsError => ApiError::InternalServerError(
                ApiErrorBody::new("Failed to get credential", "get_user_credentials_error"),
            ),
            CredentialError::DeleteCredentialError => ApiError::InternalServerError(
                ApiErrorBody::new("Failed to delete credential", "delete_credential_error"),
            ),
            CredentialError::VerifyPasswordError(error) => {
                ApiError::InternalServerError(ApiErrorBody::new(
                    format!("Failed to verify password: {error}"),
                    "verify_password_error",
                ))
            }
            CredentialError::DeletePasswordCredentialError => {
                ApiError::InternalServerError(ApiErrorBody::new(
                    "Failed to delete password credential",
                    "delete_password_credential_error",
                ))
            }
            CredentialError::GetPasswordCredentialError => {
                ApiError::InternalServerError(ApiErrorBody::new(
                    "Failed to get password credential",
                    "get_password_credential_error",
                ))
            }
            CredentialError::HashPasswordError(error) => {
                ApiError::InternalServerError(ApiErrorBody::new(
                    format!("Failed to hash password: {error}"),
                    "hash_password_error",
                ))
            }
            CredentialError::UpdateCredentialError => ApiError::InternalServerError(
                ApiErrorBody::new("Internal server error", "update_credential_error"),
            ),
            CredentialError::UnexpectedCredentialData => ApiError::InternalServerError(
                ApiErrorBody::new("Internal server error", "unexpected_credential_data"),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_entities::api_error::serialized_error;
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    async fn body_of(error: CoreError) -> Value {
        serialized_error(ApiError::from(error).into_response()).await
    }

    #[test]
    fn maps_username_already_exists_to_bad_request() {
        let error = ApiError::from(CoreError::UsernameAlreadyExists);

        assert_eq!(
            error,
            ApiError::BadRequest(ApiErrorBody::new(
                "Username already exists in this realm",
                "username_already_exists"
            ))
        );
    }

    #[tokio::test]
    async fn a_business_error_keeps_its_http_code_and_gains_a_reason() {
        let body = body_of(CoreError::EmailAlreadyExists).await;

        assert_eq!(
            body,
            json!({
                "code": "E_BAD_REQUEST",
                "status": 400,
                "reason": "email_already_exists",
                "message": "Email already exists in this realm",
            })
        );
    }

    #[tokio::test]
    async fn maps_a_taken_user_id_to_conflict() {
        let body = body_of(CoreError::UserIdAlreadyExists).await;

        assert_eq!(
            body,
            json!({
                "code": "E_CONFLICT",
                "status": 409,
                "reason": "user_id_already_exists",
                "message": "A user already exists with this id",
            })
        );
    }

    #[tokio::test]
    async fn errors_sharing_a_http_code_are_told_apart_by_their_reason() {
        let errors = [
            CoreError::EmailAlreadyExists,
            CoreError::UsernameAlreadyExists,
            CoreError::AlreadyExists,
            CoreError::Invalid,
            CoreError::InvalidRedirectUri,
        ];

        let mut reasons = Vec::new();

        for error in errors {
            let body = body_of(error).await;
            assert_eq!(body["code"], json!("E_BAD_REQUEST"));
            reasons.push(
                body["reason"]
                    .as_str()
                    .expect("every error response carries a reason")
                    .to_string(),
            );
        }

        assert_eq!(
            reasons,
            vec![
                "email_already_exists",
                "username_already_exists",
                "already_exists",
                "invalid",
                "invalid_redirect_uri",
            ]
        );
    }

    #[tokio::test]
    async fn a_password_policy_violation_reports_one_coded_entry_per_broken_rule() {
        let payload = to_string(&vec![
            PasswordPolicyViolation {
                code: "too_short".to_string(),
                message: "Password is too short: 4 characters (minimum 12 required)".to_string(),
            },
            PasswordPolicyViolation {
                code: "missing_uppercase".to_string(),
                message: "Password must contain at least one uppercase letter".to_string(),
            },
        ])
        .expect("violations serialize");

        let body = body_of(CoreError::PasswordPolicyViolation(payload)).await;

        assert_eq!(
            body,
            json!({
                "errors": [
                    {
                        "field": "password",
                        "code": "too_short",
                        "message": "Password is too short: 4 characters (minimum 12 required)",
                    },
                    {
                        "field": "password",
                        "code": "missing_uppercase",
                        "message": "Password must contain at least one uppercase letter",
                    },
                ]
            })
        );
    }

    #[tokio::test]
    async fn an_unparsable_password_policy_payload_falls_back_to_the_core_reason() {
        let body = body_of(CoreError::PasswordPolicyViolation("not json".to_string())).await;

        assert_eq!(
            body,
            json!({
                "errors": [
                    {
                        "field": "password",
                        "code": "password_policy_violation",
                        "message": "not json",
                    },
                ]
            })
        );
    }

    #[tokio::test]
    async fn an_existing_password_on_import_is_a_conflict() {
        let response = ApiError::from(CoreError::PasswordCredentialAlreadyExists).into_response();
        assert_eq!(response.status(), axum::http::StatusCode::CONFLICT);

        let body = body_of(CoreError::PasswordCredentialAlreadyExists).await;
        assert_eq!(body["reason"], json!("password_credential_already_exists"));
    }

    #[tokio::test]
    async fn an_unsupported_imported_hash_points_at_secret_data() {
        let response =
            ApiError::from(CoreError::InvalidPasswordHash("bad".to_string())).into_response();
        assert_eq!(
            response.status(),
            axum::http::StatusCode::UNPROCESSABLE_ENTITY
        );

        let body = body_of(CoreError::InvalidPasswordHash(
            "bcrypt cost 3 is outside 4..=14".to_string(),
        ))
        .await;
        assert_eq!(
            body,
            json!({
                "errors": [
                    {
                        "field": "secret_data",
                        "code": "invalid_password_hash",
                        "message": "bcrypt cost 3 is outside 4..=14",
                    },
                ]
            })
        );
    }

    #[tokio::test]
    async fn invalid_grant_keeps_its_rfc_6749_code_and_does_not_leak_a_reason() {
        let body = body_of(CoreError::InvalidGrant(
            "Refresh token is expired".to_string(),
        ))
        .await;

        assert_eq!(
            body,
            json!({
                "error": "invalid_grant",
                "error_description": "Refresh token is expired",
            })
        );
    }

    #[tokio::test]
    async fn an_invalid_authorization_code_stays_a_single_opaque_cause() {
        let body = body_of(CoreError::InvalidAuthorizationCode).await;

        assert_eq!(
            body,
            json!({
                "error": "invalid_grant",
                "error_description": "The authorization code is invalid, expired, already used, or was not issued to this client.",
            })
        );
    }

    #[tokio::test]
    async fn no_error_renders_an_empty_message() {
        let errors = [
            CoreError::AuthorizationCodeStorageFailed,
            CoreError::NotFound,
            CoreError::InternalServerError,
            CoreError::InvalidCredentials,
            CoreError::SessionRevoked,
            CoreError::UserDisabled,
            CoreError::AccountLocked,
            CoreError::RealmKeyNotFound,
            CoreError::WebhookDeliveryNotReplayable,
            CoreError::ClientUnderMaintenance("Scheduled maintenance".to_string()),
            CoreError::Forbidden("You cannot do that".to_string()),
            CoreError::ServiceUnavailable("Try again later".to_string()),
        ];

        for error in errors {
            let reason = error.reason();
            let body = body_of(error).await;
            let message = body["message"]
                .as_str()
                .expect("every error response carries a message");

            assert!(
                !message.is_empty() && !message.ends_with(": "),
                "reason {reason} renders an empty message: {message:?}"
            );
        }
    }
}

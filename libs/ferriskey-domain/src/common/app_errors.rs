use thiserror::Error;

use crate::authentication::entities::AuthenticationError;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Not found")]
    NotFound,

    #[error("Already exists")]
    AlreadyExists,

    #[error("Email already exists in this realm")]
    EmailAlreadyExists,

    #[error("Username already exists in this realm")]
    UsernameAlreadyExists,

    #[error("Invalid resource")]
    Invalid,

    #[error("Invalid required action: {0}")]
    InvalidRequiredAction(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Internal server error")]
    InternalServerError,

    #[error("No redirect URI is registered for this client")]
    RedirectUriNotFound,

    #[error("Redirect URI is not allowed for this client")]
    InvalidRedirectUri,

    #[error("No web origin is registered under this identifier")]
    WebOriginNotFound,

    #[error("Invalid web origin: {0}")]
    InvalidWebOrigin(String),

    #[error("No SAML configuration is registered for this client")]
    SamlConfigNotFound,

    #[error("Invalid SAML configuration: {0}")]
    InvalidSamlConfig(String),

    #[error("No SAML attribute mapper is registered under this identifier")]
    SamlAttributeMapperNotFound,

    #[error("Invalid SAML attribute mapper: {0}")]
    InvalidSamlAttributeMapper(String),

    #[error("Invalid client")]
    InvalidClient,

    #[error("Invalid realm")]
    InvalidRealm,

    #[error("A realm named {0} already exists")]
    RealmAlreadyExists(String),

    #[error("Invalid user")]
    InvalidUser,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Invalid state")]
    InvalidState,

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("The session backing this token no longer exists or has expired")]
    SessionRevoked,

    #[error("{0}")]
    InvalidGrant(String),

    #[error("Invalid client secret")]
    InvalidClientSecret,

    #[error("Invalid authorization request")]
    InvalidRequest,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Service account not found")]
    ServiceAccountNotFound,

    #[error("Hash password error: {0}")]
    HashPasswordError(String),

    #[error("Verify password error: {0}")]
    VerifyPasswordError(String),

    #[error("Failed to delete password credential")]
    DeletePasswordCredentialError,

    #[error("Failed to create credential")]
    CreateCredentialError,

    #[error("Failed to get password credential")]
    GetPasswordCredentialError,

    #[error("Failed to get user credentials")]
    GetUserCredentialsError,

    #[error("Failed to delete credential")]
    DeleteCredentialError,

    #[error("Token generation error: {0}")]
    TokenGenerationError(String),

    #[error("Token validation error: {0}")]
    TokenValidationError(String),

    #[error("Token parsing error: {0}")]
    TokenParsingError(String),

    #[error("Token expiration error: {0}")]
    TokenExpirationError(String),

    #[error("Realm key not found")]
    RealmKeyNotFound,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Expired token")]
    ExpiredToken,

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid session")]
    InvalidSession,

    #[error("Failed to create session")]
    SessionCreateError,

    #[error("Failed to delete session")]
    SessionDeleteError,

    #[error("Invalid TOTP secret format")]
    InvalidTotpSecretFormat,

    #[error("TOTP generation failed: {0}")]
    TotpGenerationFailed(String),

    #[error("TOTP verification failed: {0}")]
    TotpVerificationFailed(String),

    #[error("Recovery code generation failed: {0}")]
    RecoveryCodeGenError(String),

    #[error("Recovery code burning failed: {0}")]
    RecoveryCodeBurnError(String),

    #[error("Cannot delete master realm")]
    CannotDeleteMasterRealm,

    #[error("Webhook not found")]
    WebhookNotFound,

    #[error("Webhook forbidden")]
    WebhookForbidden,

    #[error("Failed to notify webhook: {0}")]
    FailedWebhookNotification(String),

    #[error("Realm not found for webhook")]
    WebhookRealmNotFound,

    #[error("Invalid webhook endpoint: {0}")]
    InvalidWebhookEndpoint(String),

    #[error("Invalid webhook retry policy: {0}")]
    InvalidWebhookRetryPolicy(String),

    #[error("Webhook delivery not found")]
    WebhookDeliveryNotFound,

    #[error("Webhook delivery is still in flight and cannot be replayed")]
    WebhookDeliveryNotReplayable,

    #[error("Failed to create client")]
    CreateClientError,

    #[error("A client with the id {0} already exists in this realm")]
    ClientIdAlreadyExists(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Authorization code storage failed")]
    AuthorizationCodeStorageFailed,

    #[error("Protocol not supported for this operation: {0}")]
    ProtocolNotSupported(String),

    #[error("Missing webauthn challenge")]
    WebAuthnMissingChallenge,

    #[error("Webauthn credential not found")]
    WebAuthnCredentialNotFound,

    #[error("Webauthn challenge failed")]
    WebAuthnChallengeFailed,

    #[error("Magic link not enabled for this realm")]
    MagicLinkNotEnabled,

    #[error("Invalid magic link token")]
    InvalidMagicLink,

    #[error("Magic link has expired")]
    MagicLinkExpired,

    #[error("Magic link has already been used")]
    MagicLinkAlreadyUsed,

    // Provider (Abyss) errors
    #[error("Provider not found")]
    ProviderNotFound,

    #[error("Provider name already exists")]
    ProviderNameAlreadyExists,

    #[error("Invalid provider configuration: {0}")]
    InvalidProviderConfiguration(String),

    #[error("Provider is disabled")]
    ProviderDisabled,

    #[error("Invalid provider URL")]
    InvalidProviderUrl,

    // Infrastructure errors
    #[error("External error: {0}")]
    External(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Federation authentication error: {0}")]
    FederationAuthenticationFailed(String),

    // Broker (SSO) errors
    #[error("Broker session not found")]
    BrokerSessionNotFound,

    #[error("Broker session expired")]
    BrokerSessionExpired,

    #[error("Invalid broker state")]
    InvalidBrokerState,

    #[error("Identity provider token exchange failed: {0}")]
    IdpTokenExchangeFailed(String),

    #[error("Identity provider userinfo failed: {0}")]
    IdpUserInfoFailed(String),

    #[error("Identity provider authentication failed: {0}")]
    IdpAuthenticationFailed(String),

    #[error("User linking failed: {0}")]
    UserLinkingFailed(String),

    #[error("Link only mode - user not found")]
    LinkOnlyUserNotFound,

    #[error("Identity provider link not found")]
    LinkNotFound,

    #[error("Invalid ID token")]
    InvalidIdToken,

    #[error("Missing authorization code")]
    MissingAuthorizationCode,

    /// Deliberately covers every authorization-code binding failure (unknown,
    /// replayed, expired, wrong client, wrong realm, redirect_uri mismatch) so
    /// the token endpoint cannot be used as an oracle to probe which of them
    /// tripped. The specific cause is logged server-side instead.
    #[error("Invalid authorization code")]
    InvalidAuthorizationCode,

    #[error("PKCE is required for this client")]
    PkceRequired,

    #[error("Invalid code_verifier")]
    InvalidCodeVerifier,

    #[error("code_challenge is required")]
    CodeChallengeMissing,

    #[error("code_verifier is required")]
    CodeVerifierMissing,

    #[error("User not found")]
    UserNotFound,

    #[error("Client not found")]
    ClientNotFound,

    #[error("Account hints not found")]
    HintsNotFound,

    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    #[error("User account is disabled")]
    UserDisabled,

    #[error("Account is temporarily locked due to too many failed login attempts")]
    AccountLocked,

    #[error("Client is under maintenance: {0}")]
    ClientUnderMaintenance(String),

    #[error("Email template not found")]
    EmailTemplateNotFound,

    #[error("No active email template for type: {0}")]
    NoActiveEmailTemplate(String),

    #[error("Invalid email template structure: {0}")]
    InvalidEmailTemplateStructure(String),

    #[error("Email template rendering failed: {0}")]
    EmailTemplateRenderError(String),

    #[error("Email verification token is invalid or expired")]
    InvalidOrExpiredToken,

    #[error("Email verification template is not configured for this realm")]
    EmailVerificationTemplateNotConfigured,

    #[error("Portal theme page tree is missing required blocks: {0}")]
    PortalThemePageInvalid(String),

    #[error("Portal theme cannot be activated: {0}")]
    PortalThemeInvalidForActivation(String),

    #[error("Portal theme is currently active and cannot be deleted")]
    PortalThemeActive,

    #[error("Portal layout is the realm default and cannot be deleted")]
    PortalLayoutDefault,

    #[error("Portal layout is referenced by one or more themes and cannot be deleted")]
    PortalLayoutInUse,

    #[error("Invalid portal layout tree: {0}")]
    PortalLayoutInvalidTree(String),

    #[error("Password policy violated: {0}")]
    PasswordPolicyViolation(String),
}

impl CoreError {
    pub fn reason(&self) -> &'static str {
        match self {
            CoreError::NotFound => "not_found",
            CoreError::AlreadyExists => "already_exists",
            CoreError::EmailAlreadyExists => "email_already_exists",
            CoreError::UsernameAlreadyExists => "username_already_exists",
            CoreError::Invalid => "invalid",
            CoreError::InvalidRequiredAction(_) => "invalid_required_action",
            CoreError::Forbidden(_) => "forbidden",
            CoreError::InternalServerError => "internal_server_error",
            CoreError::RedirectUriNotFound => "redirect_uri_not_found",
            CoreError::InvalidRedirectUri => "invalid_redirect_uri",
            CoreError::WebOriginNotFound => "web_origin_not_found",
            CoreError::InvalidWebOrigin(_) => "invalid_web_origin",
            CoreError::SamlConfigNotFound => "saml_config_not_found",
            CoreError::InvalidSamlConfig(_) => "invalid_saml_config",
            CoreError::SamlAttributeMapperNotFound => "saml_attribute_mapper_not_found",
            CoreError::InvalidSamlAttributeMapper(_) => "invalid_saml_attribute_mapper",
            CoreError::InvalidClient => "invalid_client",
            CoreError::InvalidRealm => "invalid_realm",
            CoreError::RealmAlreadyExists(_) => "realm_already_exists",
            CoreError::InvalidUser => "invalid_user",
            CoreError::InvalidPassword => "invalid_password",
            CoreError::InvalidState => "invalid_state",
            CoreError::InvalidRefreshToken => "invalid_refresh_token",
            CoreError::SessionRevoked => "session_revoked",
            CoreError::InvalidGrant(_) => "invalid_grant",
            CoreError::InvalidClientSecret => "invalid_client_secret",
            CoreError::InvalidRequest => "invalid_request",
            CoreError::InvalidCredentials => "invalid_credentials",
            CoreError::ServiceAccountNotFound => "service_account_not_found",
            CoreError::HashPasswordError(_) => "hash_password_error",
            CoreError::VerifyPasswordError(_) => "verify_password_error",
            CoreError::DeletePasswordCredentialError => "delete_password_credential_error",
            CoreError::CreateCredentialError => "create_credential_error",
            CoreError::GetPasswordCredentialError => "get_password_credential_error",
            CoreError::GetUserCredentialsError => "get_user_credentials_error",
            CoreError::DeleteCredentialError => "delete_credential_error",
            CoreError::TokenGenerationError(_) => "token_generation_error",
            CoreError::TokenValidationError(_) => "token_validation_error",
            CoreError::TokenParsingError(_) => "token_parsing_error",
            CoreError::TokenExpirationError(_) => "token_expiration_error",
            CoreError::RealmKeyNotFound => "realm_key_not_found",
            CoreError::InvalidToken => "invalid_token",
            CoreError::ExpiredToken => "expired_token",
            CoreError::InvalidKey(_) => "invalid_key",
            CoreError::SessionNotFound => "session_not_found",
            CoreError::SessionExpired => "session_expired",
            CoreError::InvalidSession => "invalid_session",
            CoreError::SessionCreateError => "session_create_error",
            CoreError::SessionDeleteError => "session_delete_error",
            CoreError::InvalidTotpSecretFormat => "invalid_totp_secret_format",
            CoreError::TotpGenerationFailed(_) => "totp_generation_failed",
            CoreError::TotpVerificationFailed(_) => "totp_verification_failed",
            CoreError::RecoveryCodeGenError(_) => "recovery_code_gen_error",
            CoreError::RecoveryCodeBurnError(_) => "recovery_code_burn_error",
            CoreError::CannotDeleteMasterRealm => "cannot_delete_master_realm",
            CoreError::WebhookNotFound => "webhook_not_found",
            CoreError::WebhookForbidden => "webhook_forbidden",
            CoreError::FailedWebhookNotification(_) => "failed_webhook_notification",
            CoreError::WebhookRealmNotFound => "webhook_realm_not_found",
            CoreError::InvalidWebhookEndpoint(_) => "invalid_webhook_endpoint",
            CoreError::InvalidWebhookRetryPolicy(_) => "invalid_webhook_retry_policy",
            CoreError::WebhookDeliveryNotFound => "webhook_delivery_not_found",
            CoreError::WebhookDeliveryNotReplayable => "webhook_delivery_not_replayable",
            CoreError::CreateClientError => "create_client_error",
            CoreError::ClientIdAlreadyExists(_) => "client_id_already_exists",
            CoreError::ServiceUnavailable(_) => "service_unavailable",
            CoreError::AuthorizationCodeStorageFailed => "authorization_code_storage_failed",
            CoreError::ProtocolNotSupported(_) => "protocol_not_supported",
            CoreError::WebAuthnMissingChallenge => "webauthn_missing_challenge",
            CoreError::WebAuthnCredentialNotFound => "webauthn_credential_not_found",
            CoreError::WebAuthnChallengeFailed => "webauthn_challenge_failed",
            CoreError::MagicLinkNotEnabled => "magic_link_not_enabled",
            CoreError::InvalidMagicLink => "invalid_magic_link",
            CoreError::MagicLinkExpired => "magic_link_expired",
            CoreError::MagicLinkAlreadyUsed => "magic_link_already_used",
            CoreError::ProviderNotFound => "provider_not_found",
            CoreError::ProviderNameAlreadyExists => "provider_name_already_exists",
            CoreError::InvalidProviderConfiguration(_) => "invalid_provider_configuration",
            CoreError::ProviderDisabled => "provider_disabled",
            CoreError::InvalidProviderUrl => "invalid_provider_url",
            CoreError::External(_) => "external",
            CoreError::Database(_) => "database",
            CoreError::Configuration(_) => "configuration",
            CoreError::FederationAuthenticationFailed(_) => "federation_authentication_failed",
            CoreError::BrokerSessionNotFound => "broker_session_not_found",
            CoreError::BrokerSessionExpired => "broker_session_expired",
            CoreError::InvalidBrokerState => "invalid_broker_state",
            CoreError::IdpTokenExchangeFailed(_) => "idp_token_exchange_failed",
            CoreError::IdpUserInfoFailed(_) => "idp_user_info_failed",
            CoreError::IdpAuthenticationFailed(_) => "idp_authentication_failed",
            CoreError::UserLinkingFailed(_) => "user_linking_failed",
            CoreError::LinkOnlyUserNotFound => "link_only_user_not_found",
            CoreError::LinkNotFound => "link_not_found",
            CoreError::InvalidIdToken => "invalid_id_token",
            CoreError::MissingAuthorizationCode => "missing_authorization_code",
            CoreError::InvalidAuthorizationCode => "invalid_authorization_code",
            CoreError::PkceRequired => "pkce_required",
            CoreError::InvalidCodeVerifier => "invalid_code_verifier",
            CoreError::CodeChallengeMissing => "code_challenge_missing",
            CoreError::CodeVerifierMissing => "code_verifier_missing",
            CoreError::UserNotFound => "user_not_found",
            CoreError::ClientNotFound => "client_not_found",
            CoreError::HintsNotFound => "hints_not_found",
            CoreError::InvalidScope(_) => "invalid_scope",
            CoreError::UserDisabled => "user_disabled",
            CoreError::AccountLocked => "account_locked",
            CoreError::ClientUnderMaintenance(_) => "client_under_maintenance",
            CoreError::EmailTemplateNotFound => "email_template_not_found",
            CoreError::NoActiveEmailTemplate(_) => "no_active_email_template",
            CoreError::InvalidEmailTemplateStructure(_) => "invalid_email_template_structure",
            CoreError::EmailTemplateRenderError(_) => "email_template_render_error",
            CoreError::InvalidOrExpiredToken => "invalid_or_expired_token",
            CoreError::EmailVerificationTemplateNotConfigured => {
                "email_verification_template_not_configured"
            }
            CoreError::PortalThemePageInvalid(_) => "portal_theme_page_invalid",
            CoreError::PortalThemeInvalidForActivation(_) => "portal_theme_invalid_for_activation",
            CoreError::PortalThemeActive => "portal_theme_active",
            CoreError::PortalLayoutDefault => "portal_layout_default",
            CoreError::PortalLayoutInUse => "portal_layout_in_use",
            CoreError::PortalLayoutInvalidTree(_) => "portal_layout_invalid_tree",
            CoreError::PasswordPolicyViolation(_) => "password_policy_violation",
        }
    }
}

impl From<AuthenticationError> for CoreError {
    fn from(err: AuthenticationError) -> Self {
        match err {
            AuthenticationError::NotFound => CoreError::SessionNotFound,
            AuthenticationError::ServiceAccountNotFound => CoreError::ServiceAccountNotFound,
            AuthenticationError::Invalid => CoreError::InvalidClient,
            AuthenticationError::InvalidRealm => CoreError::InvalidRealm,
            AuthenticationError::InvalidClient => CoreError::InvalidClient,
            AuthenticationError::InvalidUser => CoreError::InvalidUser,
            AuthenticationError::InvalidPassword => CoreError::InvalidPassword,
            AuthenticationError::InvalidState => CoreError::InvalidState,
            AuthenticationError::InvalidRefreshToken => CoreError::InvalidRefreshToken,
            AuthenticationError::InternalServerError => CoreError::InternalServerError,
            AuthenticationError::InvalidClientSecret => CoreError::InvalidClientSecret,
            AuthenticationError::InvalidRequest => CoreError::InvalidRequest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CoreError;
    use std::collections::HashMap;

    fn every_variant_with_expected_reason() -> Vec<(CoreError, &'static str)> {
        let payload = String::new();
        vec![
            (CoreError::NotFound, "not_found"),
            (CoreError::AlreadyExists, "already_exists"),
            (CoreError::EmailAlreadyExists, "email_already_exists"),
            (CoreError::UsernameAlreadyExists, "username_already_exists"),
            (CoreError::Invalid, "invalid"),
            (
                CoreError::InvalidRequiredAction(payload.clone()),
                "invalid_required_action",
            ),
            (CoreError::Forbidden(payload.clone()), "forbidden"),
            (CoreError::InternalServerError, "internal_server_error"),
            (CoreError::RedirectUriNotFound, "redirect_uri_not_found"),
            (CoreError::InvalidRedirectUri, "invalid_redirect_uri"),
            (CoreError::WebOriginNotFound, "web_origin_not_found"),
            (
                CoreError::InvalidWebOrigin(payload.clone()),
                "invalid_web_origin",
            ),
            (CoreError::SamlConfigNotFound, "saml_config_not_found"),
            (
                CoreError::InvalidSamlConfig(payload.clone()),
                "invalid_saml_config",
            ),
            (
                CoreError::SamlAttributeMapperNotFound,
                "saml_attribute_mapper_not_found",
            ),
            (
                CoreError::InvalidSamlAttributeMapper(payload.clone()),
                "invalid_saml_attribute_mapper",
            ),
            (CoreError::InvalidClient, "invalid_client"),
            (CoreError::InvalidRealm, "invalid_realm"),
            (
                CoreError::RealmAlreadyExists(payload.clone()),
                "realm_already_exists",
            ),
            (CoreError::InvalidUser, "invalid_user"),
            (CoreError::InvalidPassword, "invalid_password"),
            (CoreError::InvalidState, "invalid_state"),
            (CoreError::InvalidRefreshToken, "invalid_refresh_token"),
            (CoreError::SessionRevoked, "session_revoked"),
            (CoreError::InvalidGrant(payload.clone()), "invalid_grant"),
            (CoreError::InvalidClientSecret, "invalid_client_secret"),
            (CoreError::InvalidRequest, "invalid_request"),
            (CoreError::InvalidCredentials, "invalid_credentials"),
            (
                CoreError::ServiceAccountNotFound,
                "service_account_not_found",
            ),
            (
                CoreError::HashPasswordError(payload.clone()),
                "hash_password_error",
            ),
            (
                CoreError::VerifyPasswordError(payload.clone()),
                "verify_password_error",
            ),
            (
                CoreError::DeletePasswordCredentialError,
                "delete_password_credential_error",
            ),
            (CoreError::CreateCredentialError, "create_credential_error"),
            (
                CoreError::GetPasswordCredentialError,
                "get_password_credential_error",
            ),
            (
                CoreError::GetUserCredentialsError,
                "get_user_credentials_error",
            ),
            (CoreError::DeleteCredentialError, "delete_credential_error"),
            (
                CoreError::TokenGenerationError(payload.clone()),
                "token_generation_error",
            ),
            (
                CoreError::TokenValidationError(payload.clone()),
                "token_validation_error",
            ),
            (
                CoreError::TokenParsingError(payload.clone()),
                "token_parsing_error",
            ),
            (
                CoreError::TokenExpirationError(payload.clone()),
                "token_expiration_error",
            ),
            (CoreError::RealmKeyNotFound, "realm_key_not_found"),
            (CoreError::InvalidToken, "invalid_token"),
            (CoreError::ExpiredToken, "expired_token"),
            (CoreError::InvalidKey(payload.clone()), "invalid_key"),
            (CoreError::SessionNotFound, "session_not_found"),
            (CoreError::SessionExpired, "session_expired"),
            (CoreError::InvalidSession, "invalid_session"),
            (CoreError::SessionCreateError, "session_create_error"),
            (CoreError::SessionDeleteError, "session_delete_error"),
            (
                CoreError::InvalidTotpSecretFormat,
                "invalid_totp_secret_format",
            ),
            (
                CoreError::TotpGenerationFailed(payload.clone()),
                "totp_generation_failed",
            ),
            (
                CoreError::TotpVerificationFailed(payload.clone()),
                "totp_verification_failed",
            ),
            (
                CoreError::RecoveryCodeGenError(payload.clone()),
                "recovery_code_gen_error",
            ),
            (
                CoreError::RecoveryCodeBurnError(payload.clone()),
                "recovery_code_burn_error",
            ),
            (
                CoreError::CannotDeleteMasterRealm,
                "cannot_delete_master_realm",
            ),
            (CoreError::WebhookNotFound, "webhook_not_found"),
            (CoreError::WebhookForbidden, "webhook_forbidden"),
            (
                CoreError::FailedWebhookNotification(payload.clone()),
                "failed_webhook_notification",
            ),
            (CoreError::WebhookRealmNotFound, "webhook_realm_not_found"),
            (
                CoreError::InvalidWebhookEndpoint(payload.clone()),
                "invalid_webhook_endpoint",
            ),
            (
                CoreError::InvalidWebhookRetryPolicy(payload.clone()),
                "invalid_webhook_retry_policy",
            ),
            (
                CoreError::WebhookDeliveryNotFound,
                "webhook_delivery_not_found",
            ),
            (
                CoreError::WebhookDeliveryNotReplayable,
                "webhook_delivery_not_replayable",
            ),
            (CoreError::CreateClientError, "create_client_error"),
            (
                CoreError::ClientIdAlreadyExists(payload.clone()),
                "client_id_already_exists",
            ),
            (
                CoreError::ServiceUnavailable(payload.clone()),
                "service_unavailable",
            ),
            (
                CoreError::AuthorizationCodeStorageFailed,
                "authorization_code_storage_failed",
            ),
            (
                CoreError::ProtocolNotSupported(payload.clone()),
                "protocol_not_supported",
            ),
            (
                CoreError::WebAuthnMissingChallenge,
                "webauthn_missing_challenge",
            ),
            (
                CoreError::WebAuthnCredentialNotFound,
                "webauthn_credential_not_found",
            ),
            (
                CoreError::WebAuthnChallengeFailed,
                "webauthn_challenge_failed",
            ),
            (CoreError::MagicLinkNotEnabled, "magic_link_not_enabled"),
            (CoreError::InvalidMagicLink, "invalid_magic_link"),
            (CoreError::MagicLinkExpired, "magic_link_expired"),
            (CoreError::MagicLinkAlreadyUsed, "magic_link_already_used"),
            (CoreError::ProviderNotFound, "provider_not_found"),
            (
                CoreError::ProviderNameAlreadyExists,
                "provider_name_already_exists",
            ),
            (
                CoreError::InvalidProviderConfiguration(payload.clone()),
                "invalid_provider_configuration",
            ),
            (CoreError::ProviderDisabled, "provider_disabled"),
            (CoreError::InvalidProviderUrl, "invalid_provider_url"),
            (CoreError::External(payload.clone()), "external"),
            (CoreError::Database(payload.clone()), "database"),
            (CoreError::Configuration(payload.clone()), "configuration"),
            (
                CoreError::FederationAuthenticationFailed(payload.clone()),
                "federation_authentication_failed",
            ),
            (CoreError::BrokerSessionNotFound, "broker_session_not_found"),
            (CoreError::BrokerSessionExpired, "broker_session_expired"),
            (CoreError::InvalidBrokerState, "invalid_broker_state"),
            (
                CoreError::IdpTokenExchangeFailed(payload.clone()),
                "idp_token_exchange_failed",
            ),
            (
                CoreError::IdpUserInfoFailed(payload.clone()),
                "idp_user_info_failed",
            ),
            (
                CoreError::IdpAuthenticationFailed(payload.clone()),
                "idp_authentication_failed",
            ),
            (
                CoreError::UserLinkingFailed(payload.clone()),
                "user_linking_failed",
            ),
            (CoreError::LinkOnlyUserNotFound, "link_only_user_not_found"),
            (CoreError::LinkNotFound, "link_not_found"),
            (CoreError::InvalidIdToken, "invalid_id_token"),
            (
                CoreError::MissingAuthorizationCode,
                "missing_authorization_code",
            ),
            (
                CoreError::InvalidAuthorizationCode,
                "invalid_authorization_code",
            ),
            (CoreError::PkceRequired, "pkce_required"),
            (CoreError::InvalidCodeVerifier, "invalid_code_verifier"),
            (CoreError::CodeChallengeMissing, "code_challenge_missing"),
            (CoreError::CodeVerifierMissing, "code_verifier_missing"),
            (CoreError::UserNotFound, "user_not_found"),
            (CoreError::ClientNotFound, "client_not_found"),
            (CoreError::HintsNotFound, "hints_not_found"),
            (CoreError::InvalidScope(payload.clone()), "invalid_scope"),
            (CoreError::UserDisabled, "user_disabled"),
            (CoreError::AccountLocked, "account_locked"),
            (
                CoreError::ClientUnderMaintenance(payload.clone()),
                "client_under_maintenance",
            ),
            (CoreError::EmailTemplateNotFound, "email_template_not_found"),
            (
                CoreError::NoActiveEmailTemplate(payload.clone()),
                "no_active_email_template",
            ),
            (
                CoreError::InvalidEmailTemplateStructure(payload.clone()),
                "invalid_email_template_structure",
            ),
            (
                CoreError::EmailTemplateRenderError(payload.clone()),
                "email_template_render_error",
            ),
            (CoreError::InvalidOrExpiredToken, "invalid_or_expired_token"),
            (
                CoreError::EmailVerificationTemplateNotConfigured,
                "email_verification_template_not_configured",
            ),
            (
                CoreError::PortalThemePageInvalid(payload.clone()),
                "portal_theme_page_invalid",
            ),
            (
                CoreError::PortalThemeInvalidForActivation(payload.clone()),
                "portal_theme_invalid_for_activation",
            ),
            (CoreError::PortalThemeActive, "portal_theme_active"),
            (CoreError::PortalLayoutDefault, "portal_layout_default"),
            (CoreError::PortalLayoutInUse, "portal_layout_in_use"),
            (
                CoreError::PortalLayoutInvalidTree(payload.clone()),
                "portal_layout_invalid_tree",
            ),
            (
                CoreError::PasswordPolicyViolation(payload),
                "password_policy_violation",
            ),
        ]
    }

    #[test]
    fn reason_codes_are_stable() {
        for (error, expected) in every_variant_with_expected_reason() {
            assert_eq!(
                error.reason(),
                expected,
                "reason code changed for {error:?}; these codes are part of the public API contract"
            );
        }
    }

    #[test]
    fn reason_codes_are_unique() {
        let mut seen: HashMap<&'static str, String> = HashMap::new();

        for (error, _) in every_variant_with_expected_reason() {
            let label = format!("{error:?}");
            if let Some(previous) = seen.insert(error.reason(), label.clone()) {
                panic!(
                    "reason code {} is shared by {previous} and {label}",
                    error.reason()
                );
            }
        }
    }

    #[test]
    fn reason_codes_cover_every_variant() {
        assert_eq!(every_variant_with_expected_reason().len(), 120);
    }

    #[test]
    fn conflicting_writes_are_told_apart_by_their_reason() {
        let codes = [
            CoreError::AlreadyExists.reason(),
            CoreError::EmailAlreadyExists.reason(),
            CoreError::UsernameAlreadyExists.reason(),
            CoreError::Invalid.reason(),
            CoreError::InvalidRedirectUri.reason(),
        ];

        let mut unique = codes.to_vec();
        unique.sort_unstable();
        unique.dedup();

        assert_eq!(
            unique.len(),
            codes.len(),
            "these five errors all map to HTTP 400 and must stay distinguishable: {codes:?}"
        );
    }
}

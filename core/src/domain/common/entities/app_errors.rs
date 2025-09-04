use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Not found")]
    NotFound,

    #[error("Already exists")]
    AlreadyExists,

    #[error("Invalid resource")]
    Invalid,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Internal server error")]
    InternalServerError,

    #[error("Redirect URI not found")]
    RedirectUriNotFound,

    #[error("Invalid redirect URI")]
    InvalidRedirectUri,

    #[error("Invalid client")]
    InvalidClient,

    #[error("Invalid realm")]
    InvalidRealm,

    #[error("Invalid user")]
    InvalidUser,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Invalid state")]
    InvalidState,

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("Invalid client secret")]
    InvalidClientSecret,

    #[error("Invalid authorization request")]
    InvalidRequest,

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

    #[error("Failed to create client")]
    CreateClientError,
}

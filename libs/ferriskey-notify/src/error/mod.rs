use thiserror::Error;

/// Single error type for the `ferriskey-notify` crate.
#[derive(Debug, Error)]
pub enum NotifyError {
    // --- domain validation ---
    #[error("Invalid port {0}: must be between 1 and 65535")]
    InvalidPort(u16),

    #[error("Invalid from_email address: {0}")]
    InvalidFromEmail(String),

    #[error("Host must not be empty")]
    EmptyHost,

    // --- repository ---
    #[error("Database error: {0}")]
    Database(String),

    #[error("SMTP configuration not found for this realm")]
    ConfigurationNotFound,

    // --- secret resolution ---
    #[error("Secret not found for reference: {0}")]
    SecretNotFound(String),

    #[error("Failed to resolve secret: {0}")]
    SecretResolutionFailed(String),

    // --- mail transport ---
    #[error("Failed to send email: {0}")]
    SendFailed(String),
}

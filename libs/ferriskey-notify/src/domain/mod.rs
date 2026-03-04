use thiserror::Error;

pub mod smtp_configuration;

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

    // --- mail transport ---
    #[error("Failed to send email: {0}")]
    SendFailed(String),
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Invalid email address: {0}")]
    InvalidEmailAddress(String),

    #[error("Invalid sender name: {0}")]
    InvalidSenderName(String),

    #[error("Invalid subject: {0}")]
    InvalidSubject(String),

    #[error("At least one recipient is required")]
    MissingRecipients,

    #[error("Failed to build email message: {0}")]
    MessageBuild(String),

    #[error("Failed to send email: {0}")]
    Transport(String),
}

#[cfg(test)]
mod tests {
    use super::EmailError;

    #[test]
    fn missing_recipients_error_message_is_stable() {
        assert_eq!(
            EmailError::MissingRecipients.to_string(),
            "At least one recipient is required"
        );
    }

    #[test]
    fn transport_error_includes_context() {
        let error = EmailError::Transport("smtp timeout".to_string());

        assert_eq!(error.to_string(), "Failed to send email: smtp timeout");
    }
}

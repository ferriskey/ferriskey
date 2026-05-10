use ferriskey_mail::{
    adapters::smtp::SmtpEmailSender,
    entities::{EmailAddress, EmailFrom, EmailMessage, EmailSubject, SmtpEncryption},
    ports::EmailSender,
};

use crate::domain::{
    common::{email::EmailPort, entities::app_errors::CoreError},
    realm::entities::SmtpConfig,
};

#[derive(Debug, Clone, Default)]
pub struct SmtpEmailPort;

impl SmtpEmailPort {
    pub fn new() -> Self {
        Self
    }
}

impl EmailPort for SmtpEmailPort {
    async fn send_email(
        &self,
        config: &SmtpConfig,
        to_email: &str,
        subject: &str,
        body: &str,
        html_body: Option<String>,
    ) -> Result<(), CoreError> {
        let encryption = match &config.encryption {
            crate::domain::realm::entities::SmtpEncryption::Tls => SmtpEncryption::Tls,
            crate::domain::realm::entities::SmtpEncryption::StartTls => SmtpEncryption::StartTls,
            crate::domain::realm::entities::SmtpEncryption::None => SmtpEncryption::None,
        };

        let sender = SmtpEmailSender::with_config(
            &config.host,
            config.port,
            &config.username,
            &config.password,
            &encryption,
        )
        .map_err(|e| CoreError::External(format!("Failed to create SMTP sender: {e}")))?;

        let from_email = EmailAddress::new(config.from_email.clone())
            .map_err(|e| CoreError::External(format!("Invalid from email: {e}")))?;
        let from = EmailFrom::new(config.from_name.clone(), from_email)
            .map_err(|e| CoreError::External(format!("Invalid from: {e}")))?;
        let to = EmailAddress::new(to_email.to_string())
            .map_err(|e| CoreError::External(format!("Invalid to email: {e}")))?;
        let email_subject = EmailSubject::new(subject.to_string())
            .map_err(|e| CoreError::External(format!("Invalid subject: {e}")))?;

        let message = EmailMessage::new(from, vec![to], email_subject, body.to_string(), html_body)
            .map_err(|e| CoreError::External(format!("Failed to build email: {e}")))?;

        sender
            .send(message)
            .await
            .map_err(|e| CoreError::External(format!("Failed to send email: {e}")))?;

        Ok(())
    }
}

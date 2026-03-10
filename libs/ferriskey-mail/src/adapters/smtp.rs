use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox,
    transport::smtp::authentication::Credentials,
};

use crate::{
    entities::{EmailMessage, SmtpEncryption},
    error::EmailError,
    ports::EmailSender,
};

pub struct SmtpEmailSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpEmailSender {
    pub fn new(host: &str, username: &str, password: &str) -> Result<Self, EmailError> {
        let creds = Credentials::new(username.to_string(), password.to_string());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|e| EmailError::Transport(format!("invalid SMTP relay host: {e}")))?
            .credentials(creds)
            .build();

        Ok(Self { mailer })
    }

    pub fn with_config(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        encryption: &SmtpEncryption,
    ) -> Result<Self, EmailError> {
        let creds = Credentials::new(username.to_string(), password.to_string());

        let mailer = match encryption {
            SmtpEncryption::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                .map_err(|e| EmailError::Transport(format!("invalid SMTP relay host: {e}")))?
                .port(port)
                .credentials(creds)
                .build(),
            SmtpEncryption::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
                .map_err(|e| EmailError::Transport(format!("invalid SMTP STARTTLS host: {e}")))?
                .port(port)
                .credentials(creds)
                .build(),
            SmtpEncryption::None => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
                .port(port)
                .credentials(creds)
                .build(),
        };

        Ok(Self { mailer })
    }
}

impl EmailSender for SmtpEmailSender {
    async fn send(&self, message: EmailMessage) -> Result<(), EmailError> {
        let EmailMessage {
            from,
            to,
            subject,
            body,
            html_body: _,
        } = message;

        if to.is_empty() {
            return Err(EmailError::MissingRecipients);
        }

        let from_mailbox: Mailbox = from
            .as_mailbox()
            .parse()
            .map_err(|e| EmailError::MessageBuild(format!("invalid from mailbox: {e}")))?;

        let mut builder = Message::builder()
            .from(from_mailbox)
            .subject(subject.as_str());

        for recipient in to {
            let recipient_mailbox: Mailbox = recipient
                .as_str()
                .parse()
                .map_err(|e| EmailError::MessageBuild(format!("invalid recipient mailbox: {e}")))?;
            builder = builder.to(recipient_mailbox);
        }

        let email = builder
            .body(body)
            .map_err(|e| EmailError::MessageBuild(format!("failed to build email message: {e}")))?;

        self.mailer
            .send(email)
            .await
            .map_err(|e| EmailError::Transport(format!("failed to send email: {e}")))?;

        Ok(())
    }
}

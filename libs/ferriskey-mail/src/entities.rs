use crate::error::EmailError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailSubject(String);

impl EmailAddress {
    pub fn new(value: String) -> Result<Self, EmailError> {
        if !value.contains("@") {
            return Err(EmailError::InvalidEmailAddress(value));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl EmailSubject {
    pub fn new(value: String) -> Result<Self, EmailError> {
        if value.trim().is_empty() {
            return Err(EmailError::InvalidSubject(value));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailFrom {
    pub name: String,
    pub email: EmailAddress,
}

impl EmailFrom {
    pub fn new(name: String, email: EmailAddress) -> Result<Self, EmailError> {
        if name.trim().is_empty() {
            return Err(EmailError::InvalidSenderName(name));
        }

        Ok(Self { name, email })
    }

    pub fn as_mailbox(&self) -> String {
        format!("{} <{}>", self.name, self.email.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub from: EmailFrom,
    pub to: Vec<EmailAddress>,
    pub subject: EmailSubject,
    pub body: String,
    pub html_body: Option<String>,
}

impl EmailMessage {
    pub fn new(
        from: EmailFrom,
        to: Vec<EmailAddress>,
        subject: EmailSubject,
        body: String,
        html_body: Option<String>,
    ) -> Result<Self, EmailError> {
        if to.is_empty() {
            return Err(EmailError::MissingRecipients);
        }

        Ok(Self {
            from,
            to,
            subject,
            body,
            html_body,
        })
    }
}

pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: EmailAddress,
    pub from_name: String,
    pub encryption: SmtpEncryption,
}

pub enum SmtpEncryption {
    Tls,
    StartTls,
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_address_rejects_missing_at_sign() {
        let result = EmailAddress::new("invalid.email".to_string());

        assert!(matches!(result, Err(EmailError::InvalidEmailAddress(_))));
    }

    #[test]
    fn email_address_accepts_valid_address() {
        let result = EmailAddress::new("alice@example.com".to_string());

        assert_eq!(
            result.expect("email address should be valid").as_str(),
            "alice@example.com"
        );
    }

    #[test]
    fn email_subject_rejects_whitespace_only() {
        let result = EmailSubject::new("   ".to_string());

        assert!(matches!(result, Err(EmailError::InvalidSubject(_))));
    }

    #[test]
    fn email_from_rejects_empty_name() {
        let email =
            EmailAddress::new("alice@example.com".to_string()).expect("email should be valid");

        let result = EmailFrom::new("  ".to_string(), email);

        assert!(matches!(result, Err(EmailError::InvalidSenderName(_))));
    }

    #[test]
    fn email_from_formats_mailbox() {
        let email =
            EmailAddress::new("alice@example.com".to_string()).expect("email should be valid");
        let from = EmailFrom::new("Alice".to_string(), email).expect("sender should be valid");

        assert_eq!(from.as_mailbox(), "Alice <alice@example.com>");
    }

    #[test]
    fn email_message_requires_at_least_one_recipient() {
        let from = EmailFrom::new(
            "Alice".to_string(),
            EmailAddress::new("alice@example.com".to_string())
                .expect("sender email should be valid"),
        )
        .expect("sender should be valid");
        let subject = EmailSubject::new("Hello".to_string()).expect("subject should be valid");

        let result = EmailMessage::new(from, vec![], subject, "Body".to_string(), None);

        assert!(matches!(result, Err(EmailError::MissingRecipients)));
    }

    #[test]
    fn email_message_accepts_valid_payload() {
        let from = EmailFrom::new(
            "Alice".to_string(),
            EmailAddress::new("alice@example.com".to_string())
                .expect("sender email should be valid"),
        )
        .expect("sender should be valid");
        let to = vec![
            EmailAddress::new("bob@example.com".to_string())
                .expect("recipient email should be valid"),
        ];
        let subject = EmailSubject::new("Hello".to_string()).expect("subject should be valid");

        let message = EmailMessage::new(
            from.clone(),
            to.clone(),
            subject.clone(),
            "Body".to_string(),
            Some("<p>Body</p>".to_string()),
        )
        .expect("message should be valid");

        assert_eq!(message.from, from);
        assert_eq!(message.to, to);
        assert_eq!(message.subject, subject);
        assert_eq!(message.body, "Body");
        assert_eq!(message.html_body.as_deref(), Some("<p>Body</p>"));
    }
}

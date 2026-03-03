use ferriskey_mail::ports::EmailSender;

use crate::domain::smtp_configuration::SmtpConfiguration;

/// Factory port that builds a concrete [`EmailSender`] from a resolved
/// [`SmtpConfiguration`] and its optional plaintext password.
///
/// Decouples `ferriskey-mail` (pure transport) from `ferriskey-notify`
/// (IAM-aware orchestration): `ferriskey-mail` never imports realm types.
///
/// Implementations live in `core/src/infrastructure/notify/` and wire
/// up the concrete `SmtpEmailSender` from `ferriskey-mail`.
pub trait MailSenderFactory: Send + Sync {
    type Sender: EmailSender;

    fn build(&self, config: &SmtpConfiguration, password: Option<String>) -> Self::Sender;
}

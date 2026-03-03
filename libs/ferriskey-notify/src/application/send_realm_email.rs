use ferriskey_domain::realm::RealmId;
use ferriskey_mail::entities::EmailMessage;
use ferriskey_mail::ports::EmailSender;

use crate::error::NotifyError;
use crate::ports::mail::MailSenderFactory;
use crate::ports::repository::SmtpConfigurationRepository;
use crate::ports::secret::SecretResolver;

/// Orchestrates sending an email through the realm's configured SMTP server.
///
/// # Type parameters
///
/// - `C` — [`SmtpConfigurationRepository`]: loads the realm SMTP config.
/// - `S` — [`SecretResolver`]: resolves the opaque `password_ref` to a
///   plaintext credential (KMS, Vault, env var, …).
/// - `F` — [`MailSenderFactory`]: constructs the concrete transport once
///   the config and password are available.
///
/// No `Box<dyn …>`, no `async_trait`.
pub struct SendRealmEmailUseCase<C, S, F> {
    smtp_config_repo: C,
    secret_resolver: S,
    mail_sender_factory: F,
}

impl<C, S, F> SendRealmEmailUseCase<C, S, F>
where
    C: SmtpConfigurationRepository,
    S: SecretResolver,
    F: MailSenderFactory,
{
    pub fn new(smtp_config_repo: C, secret_resolver: S, mail_sender_factory: F) -> Self {
        Self {
            smtp_config_repo,
            secret_resolver,
            mail_sender_factory,
        }
    }

    /// Send `message` through the SMTP gateway configured for `realm_id`.
    ///
    /// # Errors
    ///
    /// - [`NotifyError::ConfigurationNotFound`] — no SMTP config exists for the realm.
    /// - [`NotifyError::Database`] — storage failure.
    /// - [`NotifyError::SecretNotFound`] / [`NotifyError::SecretResolutionFailed`] — secret backend error.
    /// - [`NotifyError::SendFailed`] — the transport rejected the message.
    pub async fn execute(
        &self,
        realm_id: &RealmId,
        message: EmailMessage,
    ) -> Result<(), NotifyError> {
        // 1. Load realm SMTP configuration.
        let config = self
            .smtp_config_repo
            .find_by_realm(realm_id)
            .await?
            .ok_or(NotifyError::ConfigurationNotFound)?;

        // 2. Resolve optional password reference.
        let password = match &config.password_ref {
            Some(secret_ref) => {
                let plaintext = self.secret_resolver.resolve(secret_ref).await?;
                Some(plaintext)
            }
            None => None,
        };

        // 3. Build the concrete mail transport and send.
        let sender = self.mail_sender_factory.build(&config, password);
        sender
            .send(message)
            .await
            .map_err(|e| NotifyError::SendFailed(e.to_string()))?;

        Ok(())
    }
}

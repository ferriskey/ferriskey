use crate::error::NotifyError;

/// Port for resolving opaque secret references to their plaintext values.
///
/// A `secret_ref` is an opaque string stored in [`SmtpConfiguration::password_ref`]
/// (e.g. `"vault://secret/smtp-password"`, `"env://SMTP_PASSWORD"`).
///
/// Concrete implementations — OpenBao, HashiCorp Vault, environment
/// variables, AWS SSM — live in `core/src/infrastructure/security/`.
///
/// [`SmtpConfiguration::password_ref`]: crate::domain::smtp_configuration::SmtpConfiguration::password_ref
pub trait SecretResolver: Send + Sync {
    fn resolve(&self, secret_ref: &str)
    -> impl Future<Output = Result<String, NotifyError>> + Send;
}

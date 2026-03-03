/// Domain entities: [`SmtpConfiguration`] and its construction input.
///
/// [`SmtpConfiguration`]: domain::smtp_configuration::SmtpConfiguration
pub mod domain;

/// Port traits: [`SmtpConfigurationRepository`], [`SecretResolver`],
/// [`MailSenderFactory`].
///
/// [`SmtpConfigurationRepository`]: ports::repository::SmtpConfigurationRepository
/// [`SecretResolver`]: ports::secret::SecretResolver
/// [`MailSenderFactory`]: ports::mail::MailSenderFactory
pub mod ports;

/// Application use cases: [`SendRealmEmailUseCase`].
///
/// [`SendRealmEmailUseCase`]: application::send_realm_email::SendRealmEmailUseCase
pub mod application;

/// Error type: [`NotifyError`].
///
/// [`NotifyError`]: error::NotifyError
pub mod error;

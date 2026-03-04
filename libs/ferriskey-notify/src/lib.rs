/// Domain entities and error type.
///
/// [`SmtpConfiguration`]: domain::smtp_configuration::SmtpConfiguration
/// [`NotifyError`]: domain::NotifyError
pub mod domain;

/// Port traits: [`SmtpConfigurationRepository`], [`SecretResolver`],
/// [`MailSenderFactory`], [`NotifyService`].
///
/// [`SmtpConfigurationRepository`]: ports::repository::SmtpConfigurationRepository
/// [`MailSenderFactory`]: ports::mail::MailSenderFactory
/// [`SmtpConfigurationService`]: ports::smtp_configuration_service::SmtpConfigurationService
pub mod ports;

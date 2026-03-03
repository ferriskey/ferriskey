use chrono::{DateTime, Utc};
use uuid::Uuid;

use ferriskey_domain::generate_uuid_v7;
use ferriskey_domain::realm::RealmId;

use crate::error::NotifyError;

/// A realm-scoped SMTP configuration.
///
/// This is a pure domain entity: no database code, no transport logic.
/// Secrets are stored as opaque references (`password_ref`) so that the
/// actual credential is only resolved at the application layer via the
/// [`SecretResolver`] port.
///
/// [`SecretResolver`]: crate::ports::secret::SecretResolver
#[derive(Debug, Clone)]
pub struct SmtpConfiguration {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    /// Opaque reference to the SMTP password (e.g. `"vault://secret/smtp"`).
    /// Resolved at runtime via [`SecretResolver`].
    ///
    /// [`SecretResolver`]: crate::ports::secret::SecretResolver
    pub password_ref: Option<String>,
    pub from_email: String,
    pub from_name: Option<String>,
    pub use_tls: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input required to construct a new [`SmtpConfiguration`].
pub struct SmtpConfigurationConfig {
    pub realm_id: RealmId,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password_ref: Option<String>,
    pub from_email: String,
    pub from_name: Option<String>,
    pub use_tls: bool,
}

impl SmtpConfiguration {
    /// Construct a validated [`SmtpConfiguration`].
    ///
    /// # Errors
    ///
    /// - [`NotifyError::InvalidPort`] — `port` is `0`.
    /// - [`NotifyError::EmptyHost`] — `host` is blank.
    /// - [`NotifyError::InvalidFromEmail`] — `from_email` has no `@`.
    pub fn new(config: SmtpConfigurationConfig) -> Result<Self, NotifyError> {
        if config.port == 0 {
            return Err(NotifyError::InvalidPort(config.port));
        }

        if config.host.trim().is_empty() {
            return Err(NotifyError::EmptyHost);
        }

        if !config.from_email.contains('@') {
            return Err(NotifyError::InvalidFromEmail(config.from_email));
        }

        let now = Utc::now();

        Ok(Self {
            id: generate_uuid_v7(),
            realm_id: config.realm_id,
            host: config.host,
            port: config.port,
            username: config.username,
            password_ref: config.password_ref,
            from_email: config.from_email,
            from_name: config.from_name,
            use_tls: config.use_tls,
            created_at: now,
            updated_at: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferriskey_domain::realm::RealmId;

    fn base_config() -> SmtpConfigurationConfig {
        SmtpConfigurationConfig {
            realm_id: RealmId::default(),
            host: "smtp.example.com".to_string(),
            port: 587,
            username: Some("user@example.com".to_string()),
            password_ref: Some("vault://secret/smtp".to_string()),
            from_email: "noreply@example.com".to_string(),
            from_name: Some("FerrisKey".to_string()),
            use_tls: true,
        }
    }

    #[test]
    fn valid_config_creates_entity() {
        let config = base_config();
        let entity = SmtpConfiguration::new(config);
        assert!(entity.is_ok());
    }

    #[test]
    fn rejects_port_zero() {
        let mut config = base_config();
        config.port = 0;
        assert!(matches!(
            SmtpConfiguration::new(config),
            Err(NotifyError::InvalidPort(0))
        ));
    }

    #[test]
    fn rejects_empty_host() {
        let mut config = base_config();
        config.host = "   ".to_string();
        assert!(matches!(
            SmtpConfiguration::new(config),
            Err(NotifyError::EmptyHost)
        ));
    }

    #[test]
    fn rejects_invalid_from_email() {
        let mut config = base_config();
        config.from_email = "not-an-email".to_string();
        assert!(matches!(
            SmtpConfiguration::new(config),
            Err(NotifyError::InvalidFromEmail(_))
        ));
    }

    #[test]
    fn port_max_value_is_accepted() {
        let mut config = base_config();
        config.port = u16::MAX; // 65535
        assert!(SmtpConfiguration::new(config).is_ok());
    }
}

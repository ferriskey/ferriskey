use ferriskey_domain::realm::RealmId;

use crate::domain::NotifyError;
use crate::domain::smtp_configuration::{SmtpConfiguration, SmtpConfigurationConfig};

/// Service port for managing realm SMTP configurations.
///
/// Implemented in `core`. Consumed by the API layer to expose
/// SMTP configuration endpoints.
pub trait SmtpConfigurationService: Send + Sync {
    /// Return the SMTP configuration for the given realm, if one exists.
    fn find_by_realm(
        &self,
        realm_id: &RealmId,
    ) -> impl Future<Output = Result<Option<SmtpConfiguration>, NotifyError>> + Send;

    /// Create or update the SMTP configuration for the given realm.
    fn upsert(
        &self,
        realm_id: &RealmId,
        config: SmtpConfigurationConfig,
    ) -> impl Future<Output = Result<SmtpConfiguration, NotifyError>> + Send;

    /// Remove the SMTP configuration for the given realm.
    fn delete(&self, realm_id: &RealmId) -> impl Future<Output = Result<(), NotifyError>> + Send;
}

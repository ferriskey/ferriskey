use ferriskey_domain::realm::RealmId;

use crate::domain::smtp_configuration::SmtpConfiguration;
use crate::error::NotifyError;

/// Storage port for [`SmtpConfiguration`] entities.
///
/// `realm_id` is a filter parameter, not a design axis: this trait
/// represents SMTP-configuration storage in general.
///
/// Infrastructure implementations (SQLx, in-memory, …) live in
/// `core/src/infrastructure/notify/`.
pub trait SmtpConfigurationRepository: Send + Sync {
    /// Return the SMTP configuration for the given realm, if one exists.
    fn find_by_realm(
        &self,
        realm_id: &RealmId,
    ) -> impl Future<Output = Result<Option<SmtpConfiguration>, NotifyError>> + Send;

    /// Insert or update the SMTP configuration for a realm.
    ///
    /// Uses the entity's `realm_id` as the natural key.
    fn upsert(
        &self,
        config: &SmtpConfiguration,
    ) -> impl Future<Output = Result<(), NotifyError>> + Send;
}

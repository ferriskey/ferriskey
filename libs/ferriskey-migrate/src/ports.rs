use std::pin::Pin;

use crate::{entities::MigrationRecord, error::MigrationError};

pub type MigrationFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'a>>;

/// Persistence port for tracking which migrations have been applied.
///
/// Infrastructure implementations (e.g. PostgreSQL via SeaORM) live in
/// `core/src/infrastructure/migrate/`.
#[cfg_attr(test, mockall::automock)]
pub trait MigrationRepository: Send + Sync {
    /// Returns the version numbers of all migrations that have already been applied.
    fn find_applied_versions(
        &self,
    ) -> impl Future<Output = Result<Vec<u64>, MigrationError>> + Send;

    /// Persists a record marking `record.version` as successfully applied.
    fn record_applied(
        &self,
        record: MigrationRecord,
    ) -> impl Future<Output = Result<(), MigrationError>> + Send;
}

/// A single data migration.
///
/// `Ctx` is the shared execution context provided by the infrastructure layer
/// (e.g. a struct holding database connections or domain services).
///
/// Implementations return a boxed future so that heterogeneous migrations can
/// be stored together in a `Vec<Box<dyn Migration<Ctx>>>`.
pub trait Migration<Ctx>: Send + Sync {
    /// Monotonically increasing version number. Must be unique across all registered migrations.
    fn version(&self) -> u64;

    /// Human-readable name, used in logs and stored in `MigrationRecord`.
    fn name(&self) -> &str;

    /// Executes the migration against `ctx`.
    fn up<'a>(&'a self, ctx: &'a Ctx) -> MigrationFuture<'a>;
}

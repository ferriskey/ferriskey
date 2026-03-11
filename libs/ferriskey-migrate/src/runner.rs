use std::collections::HashSet;

use chrono::Utc;

use crate::{
    entities::{MigrationRecord, MigrationReport},
    error::MigrationError,
    ports::{Migration, MigrationRepository},
};

/// Orchestrates execution of pending data migrations.
///
/// Build with [`MigrationRunner::new`], register migrations via [`register`](Self::register),
/// then call [`run`](Self::run) at application startup.
///
/// ```rust,ignore
/// let report = MigrationRunner::new(repo)
///     .register(BackfillUserSlugs)
///     .register(NormaliseRoleNames)
///     .run(&ctx)
///     .await?;
/// ```
pub struct MigrationRunner<Ctx, R> {
    migrations: Vec<Box<dyn Migration<Ctx>>>,
    repository: R,
}

impl<Ctx: Send + Sync, R: MigrationRepository> MigrationRunner<Ctx, R> {
    pub fn new(repository: R) -> Self {
        Self {
            migrations: Vec::new(),
            repository,
        }
    }

    /// Appends a migration to the runner. Migrations are executed in ascending version order
    /// regardless of registration order.
    pub fn register(mut self, migration: impl Migration<Ctx> + 'static) -> Self {
        self.migrations.push(Box::new(migration));
        self
    }

    /// Runs all pending migrations against `ctx`.
    ///
    /// Skips any migration whose version is already present in the repository.
    /// Stops on the first failure, leaving subsequent migrations unapplied.
    pub async fn run(&self, ctx: &Ctx) -> Result<MigrationReport, MigrationError> {
        self.assert_no_duplicate_versions()?;

        let applied_versions: HashSet<u64> = self
            .repository
            .find_applied_versions()
            .await?
            .into_iter()
            .collect();

        let mut pending: Vec<&Box<dyn Migration<Ctx>>> = self
            .migrations
            .iter()
            .filter(|m| !applied_versions.contains(&m.version()))
            .collect();

        pending.sort_by_key(|m| m.version());

        let mut report = MigrationReport::default();

        for migration in pending {
            migration
                .up(ctx)
                .await
                .map_err(|e| MigrationError::ExecutionFailed {
                    version: migration.version(),
                    name: migration.name().to_string(),
                    reason: e.to_string(),
                })?;

            let record = MigrationRecord {
                version: migration.version(),
                name: migration.name().to_string(),
                applied_at: Utc::now(),
            };

            self.repository
                .record_applied(record.clone())
                .await
                .map_err(|e| MigrationError::RecordFailed {
                    version: migration.version(),
                    reason: e.to_string(),
                })?;

            report.applied.push(record);
        }

        Ok(report)
    }

    fn assert_no_duplicate_versions(&self) -> Result<(), MigrationError> {
        let mut seen = HashSet::new();
        for m in &self.migrations {
            if !seen.insert(m.version()) {
                return Err(MigrationError::DuplicateVersion(m.version()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{MigrationFuture, MockMigrationRepository};

    struct NoopMigration {
        version: u64,
        name: &'static str,
    }

    impl Migration<()> for NoopMigration {
        fn version(&self) -> u64 {
            self.version
        }

        fn name(&self) -> &str {
            self.name
        }

        fn up<'a>(&'a self, _ctx: &'a ()) -> MigrationFuture<'a> {
            Box::pin(async { Ok(()) })
        }
    }

    struct FailingMigration {
        version: u64,
    }

    impl Migration<()> for FailingMigration {
        fn version(&self) -> u64 {
            self.version
        }

        fn name(&self) -> &str {
            "failing_migration"
        }

        fn up<'a>(&'a self, _ctx: &'a ()) -> MigrationFuture<'a> {
            Box::pin(async { Err("something went wrong".into()) })
        }
    }

    #[tokio::test]
    async fn skips_already_applied_migrations() {
        let mut repo = MockMigrationRepository::new();
        repo.expect_find_applied_versions()
            .returning(|| Box::pin(async { Ok(vec![1]) }));

        let report = MigrationRunner::new(repo)
            .register(NoopMigration {
                version: 1,
                name: "already_done",
            })
            .run(&())
            .await
            .expect("runner should succeed");

        assert!(report.is_empty());
    }

    #[tokio::test]
    async fn applies_pending_migrations_in_version_order() {
        let mut repo = MockMigrationRepository::new();
        repo.expect_find_applied_versions()
            .returning(|| Box::pin(async { Ok(vec![]) }));
        repo.expect_record_applied()
            .times(2)
            .returning(|_| Box::pin(async { Ok(()) }));

        let report = MigrationRunner::new(repo)
            // Registered out of order intentionally.
            .register(NoopMigration {
                version: 2,
                name: "second",
            })
            .register(NoopMigration {
                version: 1,
                name: "first",
            })
            .run(&())
            .await
            .expect("runner should succeed");

        assert_eq!(report.applied.len(), 2);
        assert_eq!(report.applied[0].version, 1);
        assert_eq!(report.applied[1].version, 2);
    }

    #[tokio::test]
    async fn returns_error_on_duplicate_version() {
        let repo = MockMigrationRepository::new();

        let result = MigrationRunner::new(repo)
            .register(NoopMigration {
                version: 1,
                name: "first",
            })
            .register(NoopMigration {
                version: 1,
                name: "duplicate",
            })
            .run(&())
            .await;

        assert!(matches!(result, Err(MigrationError::DuplicateVersion(1))));
    }

    #[tokio::test]
    async fn stops_on_first_failure() {
        let mut repo = MockMigrationRepository::new();
        repo.expect_find_applied_versions()
            .returning(|| Box::pin(async { Ok(vec![]) }));
        // record_applied must never be called when execution fails.
        repo.expect_record_applied().never();

        let result = MigrationRunner::new(repo)
            .register(FailingMigration { version: 1 })
            .run(&())
            .await;

        assert!(matches!(
            result,
            Err(MigrationError::ExecutionFailed { version: 1, .. })
        ));
    }

    #[tokio::test]
    async fn records_migration_after_successful_execution() {
        let mut repo = MockMigrationRepository::new();
        repo.expect_find_applied_versions()
            .returning(|| Box::pin(async { Ok(vec![]) }));
        repo.expect_record_applied()
            .times(1)
            .withf(|r| r.version == 1 && r.name == "backfill_slugs")
            .returning(|_| Box::pin(async { Ok(()) }));

        MigrationRunner::new(repo)
            .register(NoopMigration {
                version: 1,
                name: "backfill_slugs",
            })
            .run(&())
            .await
            .expect("runner should succeed");
    }
}

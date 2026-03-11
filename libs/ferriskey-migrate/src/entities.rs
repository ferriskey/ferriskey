use chrono::{DateTime, Utc};

/// Persistent record of a migration that has been successfully applied.
#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub version: u64,
    pub name: String,
    pub applied_at: DateTime<Utc>,
}

/// Summary of a single `MigrationRunner::run` call.
#[derive(Debug, Default)]
pub struct MigrationReport {
    /// Migrations applied during this run, in ascending version order.
    pub applied: Vec<MigrationRecord>,
}

impl MigrationReport {
    /// Returns `true` when no migrations were applied (everything was already up to date).
    pub fn is_empty(&self) -> bool {
        self.applied.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_is_empty_when_no_migrations_applied() {
        let report = MigrationReport::default();

        assert!(report.is_empty());
    }

    #[test]
    fn report_is_not_empty_after_adding_record() {
        let mut report = MigrationReport::default();
        report.applied.push(MigrationRecord {
            version: 1,
            name: "backfill_slugs".to_string(),
            applied_at: Utc::now(),
        });

        assert!(!report.is_empty());
    }
}

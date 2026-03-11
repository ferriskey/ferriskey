use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("Migration v{version} '{name}' failed: {reason}")]
    ExecutionFailed {
        version: u64,
        name: String,
        reason: String,
    },

    #[error("Failed to load applied migrations: {0}")]
    LoadFailed(String),

    #[error("Failed to record migration v{version}: {reason}")]
    RecordFailed { version: u64, reason: String },

    #[error("Duplicate migration version: {0}")]
    DuplicateVersion(u64),
}

#[cfg(test)]
mod tests {
    use super::MigrationError;

    #[test]
    fn execution_failed_includes_version_name_and_reason() {
        let err = MigrationError::ExecutionFailed {
            version: 1,
            name: "backfill_user_slugs".to_string(),
            reason: "connection refused".to_string(),
        };

        assert_eq!(
            err.to_string(),
            "Migration v1 'backfill_user_slugs' failed: connection refused"
        );
    }

    #[test]
    fn duplicate_version_error_message_is_stable() {
        let err = MigrationError::DuplicateVersion(42);

        assert_eq!(err.to_string(), "Duplicate migration version: 42");
    }
}

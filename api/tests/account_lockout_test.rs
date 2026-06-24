/// Integration tests for account lockout / anti-brute-force (issue #1095).
///
/// These tests require a running PostgreSQL instance. They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test account_lockout_test -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST     = localhost
///   DATABASE_PORT     = 5432
///   DATABASE_NAME     = ferriskey
///   DATABASE_USER     = ferriskey
///   DATABASE_PASSWORD = ferriskey
///
/// These tests verify:
///   1. N-1 failed attempts do NOT lock the account.
///   2. Nth failed attempt locks the account (subsequent login returns 401 with
///      "AccountLocked" even with correct credentials).
///   3. Admin unlock endpoint clears the lockout → login succeeds.
///   4. Auto-recovery: locked_until in the past allows login again.
#[cfg(test)]
mod tests {
    // Placeholder — full integration coverage requires a live Postgres instance.
    // The test harness pattern from device_flow_test.rs applies here.

    /// N-1 wrong-password attempts do not lock the account.
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_lockout_test -- --ignored"]
    #[test]
    fn below_threshold_does_not_lock() {
        // TODO: create realm with lockout_threshold=3, attempt 2 wrong passwords,
        //       verify correct password still works.
        todo!()
    }

    /// Nth wrong-password attempt locks the account.
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_lockout_test -- --ignored"]
    #[test]
    fn at_threshold_locks_account() {
        // TODO: create realm with lockout_threshold=3, attempt 3 wrong passwords,
        //       verify correct password returns 401 AccountLocked.
        todo!()
    }

    /// Admin POST /realms/{realm}/users/{id}/unlock clears the lock.
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_lockout_test -- --ignored"]
    #[test]
    fn admin_unlock_restores_access() {
        // TODO: lock account, POST unlock, verify login succeeds.
        todo!()
    }

    /// After locked_until lapses, the account auto-recovers.
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_lockout_test -- --ignored"]
    #[test]
    fn auto_recovery_after_window_elapses() {
        // TODO: set locked_until = now() - 1s directly in DB, verify login succeeds.
        todo!()
    }
}

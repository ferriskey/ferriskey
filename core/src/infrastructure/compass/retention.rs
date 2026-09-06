use std::time::Duration as StdDuration;

use chrono::{Duration, Utc};
use ferriskey_compass::ports::CompassFlowRepository;

/// How long a traced flow is kept before it is deleted.
pub const FLOW_RETENTION_DAYS: i64 = 30;

/// How long a flow may sit in `pending` before it is considered abandoned.
///
/// A login the user walked away from never completes, so without this every
/// abandoned `/auth` hit would count as an in-progress login forever. Sized
/// well above any realistic interactive login, including MFA and a detour
/// through an external IdP.
pub const PENDING_FLOW_TIMEOUT_HOURS: i64 = 12;

const SWEEP_INTERVAL: StdDuration = StdDuration::from_secs(60 * 60);

/// Applies the Compass retention window: expires abandoned flows, then deletes
/// flows past the retention horizon.
///
/// Failures are logged and the loop continues — a dashboard falling behind on
/// housekeeping must not take the process down.
pub async fn compass_retention_task<F>(flow_repo: F)
where
    F: CompassFlowRepository,
{
    let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
    // The first tick fires immediately; on a long-running deployment that is
    // the sweep that clears whatever accumulated while the process was down.
    loop {
        ticker.tick().await;

        let now = Utc::now();

        match flow_repo
            .expire_stale_flows(now - Duration::hours(PENDING_FLOW_TIMEOUT_HOURS))
            .await
        {
            Ok(0) => {}
            Ok(count) => tracing::info!(count, "Compass retention: expired abandoned flows"),
            Err(e) => tracing::error!("Compass retention: failed to expire stale flows: {e}"),
        }

        match flow_repo
            .purge_old_flows(now - Duration::days(FLOW_RETENTION_DAYS))
            .await
        {
            Ok(0) => {}
            Ok(count) => tracing::info!(
                count,
                retention_days = FLOW_RETENTION_DAYS,
                "Compass retention: purged flows past the retention window"
            ),
            Err(e) => tracing::error!("Compass retention: failed to purge old flows: {e}"),
        }
    }
}

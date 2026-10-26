use std::num::NonZeroUsize;
use std::sync::Arc;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

const REQUESTS_PER_CPU: usize = 32;

/// Caps the requests of one kind served at once, so a saturated server turns
/// the surplus away at once instead of queuing it until every caller times out.
#[derive(Debug, Clone)]
pub struct Admission {
    permits: Arc<Semaphore>,
}

impl Admission {
    pub fn new(limit: NonZeroUsize) -> Self {
        Self {
            permits: Arc::new(Semaphore::new(limit.get())),
        }
    }

    /// The configured limit, or a share of the CPUs the process may use: past
    /// that, requests only wait for a core and add latency without any throughput.
    pub fn sized(configured: Option<NonZeroUsize>) -> Self {
        let cpus = std::thread::available_parallelism().map_or(1, usize::from);
        let per_cpu = NonZeroUsize::new(cpus * REQUESTS_PER_CPU).unwrap_or(NonZeroUsize::MIN);
        Self::new(configured.unwrap_or(per_cpu))
    }

    pub fn try_enter(&self) -> Option<OwnedSemaphorePermit> {
        Arc::clone(&self.permits).try_acquire_owned().ok()
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::Admission;

    fn limit(value: usize) -> NonZeroUsize {
        NonZeroUsize::new(value).unwrap_or(NonZeroUsize::MIN)
    }

    #[test]
    fn requests_beyond_the_limit_are_refused_until_one_leaves() {
        let admission = Admission::new(limit(2));

        let first = admission.try_enter();
        let second = admission.try_enter();
        assert!(first.is_some() && second.is_some());
        assert!(admission.try_enter().is_none());

        drop(first);
        assert!(admission.try_enter().is_some());
    }

    #[test]
    fn a_configured_limit_wins_over_the_cpu_count() {
        let admission = Admission::sized(Some(limit(1)));

        let held = admission.try_enter();
        assert!(held.is_some());
        assert!(admission.try_enter().is_none());
    }
}

use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use uuid::Uuid;

pub const DEFAULT_REQUESTS_PER_MINUTE: u32 = 10;
pub const CIMD_FETCHES_PER_MINUTE: u32 = 30;
pub const REALM_REGISTRATIONS_PER_HOUR: u32 = 60;
pub const MAX_TRACKED_ADDRESSES: usize = 10_000;
pub const MAX_TRACKED_REALMS: usize = 10_000;
const REALM_BUDGET_WINDOW: Duration = Duration::from_secs(60 * 60);

fn rate_limit_key(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(_) => ip,
        IpAddr::V6(v6) => {
            let mut octets = v6.octets();
            octets[8..].fill(0);
            IpAddr::V6(Ipv6Addr::from(octets))
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Bucket {
    tokens: f64,
    refilled_at: Instant,
}

#[derive(Debug)]
pub struct IpRateLimiter {
    capacity: f64,
    refill_per_second: f64,
    max_tracked: usize,
    buckets: Mutex<HashMap<IpAddr, Bucket>>,
}

impl IpRateLimiter {
    pub fn per_minute(requests: u32) -> Self {
        Self::with_limits(requests, MAX_TRACKED_ADDRESSES)
    }

    pub fn with_limits(requests_per_minute: u32, max_tracked: usize) -> Self {
        let capacity = f64::from(requests_per_minute.max(1));
        Self {
            capacity,
            refill_per_second: capacity / 60.0,
            max_tracked: max_tracked.max(1),
            buckets: Mutex::new(HashMap::new()),
        }
    }

    pub fn allow(&self, ip: IpAddr) -> bool {
        self.allow_at(ip, Instant::now())
    }

    fn allow_at(&self, ip: IpAddr, now: Instant) -> bool {
        let ip = rate_limit_key(ip);
        let Ok(mut buckets) = self.buckets.lock() else {
            return false;
        };

        if !buckets.contains_key(&ip) && buckets.len() >= self.max_tracked {
            self.make_room(&mut buckets, now);
        }

        let bucket = buckets.entry(ip).or_insert(Bucket {
            tokens: self.capacity,
            refilled_at: now,
        });

        let elapsed = now.saturating_duration_since(bucket.refilled_at);
        bucket.tokens =
            (bucket.tokens + elapsed.as_secs_f64() * self.refill_per_second).min(self.capacity);
        bucket.refilled_at = now;

        if bucket.tokens < 1.0 {
            return false;
        }

        bucket.tokens -= 1.0;
        true
    }

    fn make_room(&self, buckets: &mut HashMap<IpAddr, Bucket>, now: Instant) {
        let full_again = Duration::from_secs_f64(self.capacity / self.refill_per_second);
        buckets.retain(|_, bucket| now.saturating_duration_since(bucket.refilled_at) < full_again);

        if buckets.len() < self.max_tracked {
            return;
        }

        let stalest = buckets
            .iter()
            .min_by_key(|(_, bucket)| bucket.refilled_at)
            .map(|(ip, _)| *ip);
        if let Some(ip) = stalest {
            buckets.remove(&ip);
        }
    }

    #[cfg(test)]
    fn tracked(&self) -> usize {
        self.buckets.lock().map(|b| b.len()).unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct RealmBudget {
    limit: u32,
    window: Duration,
    max_tracked: usize,
    windows: Mutex<HashMap<Uuid, (Instant, u32)>>,
}

impl RealmBudget {
    pub fn per_hour(limit: u32) -> Self {
        Self::with_limits(limit, REALM_BUDGET_WINDOW, MAX_TRACKED_REALMS)
    }

    pub fn with_limits(limit: u32, window: Duration, max_tracked: usize) -> Self {
        Self {
            limit: limit.max(1),
            window,
            max_tracked: max_tracked.max(1),
            windows: Mutex::new(HashMap::new()),
        }
    }

    pub fn allow(&self, realm: Uuid) -> bool {
        self.allow_at(realm, Instant::now())
    }

    fn allow_at(&self, realm: Uuid, now: Instant) -> bool {
        let Ok(mut windows) = self.windows.lock() else {
            return false;
        };

        if !windows.contains_key(&realm) && windows.len() >= self.max_tracked {
            let window = self.window;
            windows.retain(|_, (started, _)| now.saturating_duration_since(*started) < window);
            if windows.len() >= self.max_tracked {
                return false;
            }
        }

        let entry = windows.entry(realm).or_insert((now, 0));
        if now.saturating_duration_since(entry.0) >= self.window {
            *entry = (now, 0);
        }

        if entry.1 >= self.limit {
            return false;
        }

        entry.1 += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    fn ip(last: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(203, 0, 113, last))
    }

    #[test]
    fn requests_beyond_the_burst_are_refused() {
        let limiter = IpRateLimiter::per_minute(10);
        let now = Instant::now();

        for _ in 0..10 {
            assert!(limiter.allow_at(ip(1), now));
        }
        assert!(!limiter.allow_at(ip(1), now));
    }

    #[test]
    fn each_address_has_its_own_bucket() {
        let limiter = IpRateLimiter::per_minute(1);
        let now = Instant::now();

        assert!(limiter.allow_at(ip(1), now));
        assert!(!limiter.allow_at(ip(1), now));
        assert!(limiter.allow_at(ip(2), now));
    }

    #[test]
    fn tokens_come_back_over_time() {
        let limiter = IpRateLimiter::per_minute(6);
        let now = Instant::now();

        for _ in 0..6 {
            assert!(limiter.allow_at(ip(1), now));
        }
        assert!(!limiter.allow_at(ip(1), now));
        assert!(limiter.allow_at(ip(1), now + Duration::from_secs(10)));
    }

    #[test]
    fn the_table_never_outgrows_its_bound() {
        let limiter = IpRateLimiter::with_limits(10, 3);
        let now = Instant::now();

        for last in 1..=20 {
            limiter.allow_at(ip(last), now);
        }

        assert!(limiter.tracked() <= 3);
    }

    #[test]
    fn ipv6_addresses_share_a_bucket_per_64() {
        let limiter = IpRateLimiter::per_minute(1);
        let now = Instant::now();
        let first: IpAddr = "2001:4860:4860:1:aaaa::1".parse().expect("must parse");
        let same_64: IpAddr = "2001:4860:4860:1:bbbb::2".parse().expect("must parse");
        let other_64: IpAddr = "2001:4860:4860:2::1".parse().expect("must parse");

        assert!(limiter.allow_at(first, now));
        assert!(!limiter.allow_at(same_64, now));
        assert!(limiter.allow_at(other_64, now));
        assert_eq!(limiter.tracked(), 2);
    }

    #[test]
    fn ipv4_addresses_are_not_merged() {
        let limiter = IpRateLimiter::per_minute(1);
        let now = Instant::now();

        assert!(limiter.allow_at(ip(1), now));
        assert!(limiter.allow_at(ip(2), now));
    }

    #[test]
    fn a_realm_budget_is_spent_across_all_callers() {
        let budget = RealmBudget::per_hour(3);
        let realm = Uuid::new_v4();
        let now = Instant::now();

        for _ in 0..3 {
            assert!(budget.allow_at(realm, now));
        }
        assert!(!budget.allow_at(realm, now));
        assert!(budget.allow_at(Uuid::new_v4(), now));
    }

    #[test]
    fn a_realm_budget_resets_after_its_window() {
        let budget = RealmBudget::per_hour(1);
        let realm = Uuid::new_v4();
        let now = Instant::now();

        assert!(budget.allow_at(realm, now));
        assert!(!budget.allow_at(realm, now + Duration::from_secs(60)));
        assert!(budget.allow_at(realm, now + REALM_BUDGET_WINDOW));
    }

    #[test]
    fn the_realm_table_never_outgrows_its_bound() {
        let budget = RealmBudget::with_limits(5, Duration::from_secs(60), 2);
        let now = Instant::now();

        assert!(budget.allow_at(Uuid::new_v4(), now));
        assert!(budget.allow_at(Uuid::new_v4(), now));
        assert!(!budget.allow_at(Uuid::new_v4(), now));
        assert!(budget.allow_at(Uuid::new_v4(), now + Duration::from_secs(61)));
    }
}

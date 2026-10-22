use std::collections::HashMap;
use std::hash::Hash;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const DEFAULT_REQUESTS_PER_MINUTE: u32 = 10;
pub const CIMD_FETCHES_PER_MINUTE: u32 = 30;
pub const REALM_REGISTRATIONS_PER_HOUR: u32 = 60;
pub const CLIENT_AUTH_FAILURES_PER_MINUTE: u32 = 10;
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
pub struct KeyedRateLimiter<K> {
    capacity: f64,
    refill_per_second: f64,
    max_tracked: usize,
    buckets: Mutex<HashMap<K, Bucket>>,
}

impl<K: Eq + Hash + Clone> KeyedRateLimiter<K> {
    pub fn with_limits(requests_per_minute: u32, max_tracked: usize) -> Self {
        let capacity = f64::from(requests_per_minute.max(1));
        Self {
            capacity,
            refill_per_second: capacity / 60.0,
            max_tracked: max_tracked.max(1),
            buckets: Mutex::new(HashMap::new()),
        }
    }

    pub fn spend_at(&self, key: K, now: Instant) -> bool {
        let Ok(mut buckets) = self.buckets.lock() else {
            return false;
        };

        if !buckets.contains_key(&key)
            && buckets.len() >= self.max_tracked
            && !self.make_room(&mut buckets, now)
        {
            return true;
        }

        let bucket = buckets.entry(key).or_insert(Bucket {
            tokens: self.capacity,
            refilled_at: now,
        });
        self.refill(bucket, now);

        if bucket.tokens < 1.0 {
            return false;
        }

        bucket.tokens -= 1.0;
        true
    }

    pub fn has_budget_at(&self, key: &K, now: Instant) -> bool {
        let Ok(buckets) = self.buckets.lock() else {
            return false;
        };
        let Some(bucket) = buckets.get(key) else {
            return true;
        };

        self.has_token(bucket, now)
    }

    fn has_token(&self, bucket: &Bucket, now: Instant) -> bool {
        let mut bucket = *bucket;
        self.refill(&mut bucket, now);
        bucket.tokens >= 1.0
    }

    fn refill(&self, bucket: &mut Bucket, now: Instant) {
        let elapsed = now.saturating_duration_since(bucket.refilled_at);
        bucket.tokens =
            (bucket.tokens + elapsed.as_secs_f64() * self.refill_per_second).min(self.capacity);
        bucket.refilled_at = now;
    }

    fn make_room(&self, buckets: &mut HashMap<K, Bucket>, now: Instant) -> bool {
        let full_again = Duration::from_secs_f64(self.capacity / self.refill_per_second);
        buckets.retain(|_, bucket| now.saturating_duration_since(bucket.refilled_at) < full_again);

        if buckets.len() < self.max_tracked {
            return true;
        }

        let stalest = buckets
            .iter()
            .filter(|(_, bucket)| self.has_token(bucket, now))
            .min_by_key(|(_, bucket)| bucket.refilled_at)
            .map(|(key, _)| key.clone());

        match stalest {
            Some(key) => {
                buckets.remove(&key);
                true
            }
            None => false,
        }
    }

    #[cfg(test)]
    fn tracked(&self) -> usize {
        self.buckets.lock().map(|b| b.len()).unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct IpRateLimiter {
    inner: KeyedRateLimiter<IpAddr>,
}

impl IpRateLimiter {
    pub fn per_minute(requests: u32) -> Self {
        Self::with_limits(requests, MAX_TRACKED_ADDRESSES)
    }

    pub fn with_limits(requests_per_minute: u32, max_tracked: usize) -> Self {
        Self {
            inner: KeyedRateLimiter::with_limits(requests_per_minute, max_tracked),
        }
    }

    pub fn allow(&self, ip: IpAddr) -> bool {
        self.allow_at(ip, Instant::now())
    }

    fn allow_at(&self, ip: IpAddr, now: Instant) -> bool {
        self.inner.spend_at(rate_limit_key(ip), now)
    }

    #[cfg(test)]
    fn tracked(&self) -> usize {
        self.inner.tracked()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThrottleKey {
    ip: IpAddr,
    identity: [u8; 32],
}

impl ThrottleKey {
    pub fn new(ip: IpAddr, realm: &str, client_id: &str) -> Self {
        let mut digest = Sha256::new();
        for part in [realm, client_id] {
            digest.update((part.len() as u64).to_be_bytes());
            digest.update(part.as_bytes());
        }

        Self {
            ip: rate_limit_key(ip),
            identity: digest.finalize().into(),
        }
    }
}

#[derive(Debug)]
pub struct ClientAuthThrottle {
    failures: KeyedRateLimiter<ThrottleKey>,
}

impl ClientAuthThrottle {
    pub fn per_minute(failures: u32) -> Self {
        Self::with_limits(failures, MAX_TRACKED_ADDRESSES)
    }

    pub fn with_limits(failures_per_minute: u32, max_tracked: usize) -> Self {
        Self {
            failures: KeyedRateLimiter::with_limits(failures_per_minute, max_tracked),
        }
    }

    pub fn reserve(&self, key: ThrottleKey) -> Option<ClientAuthAttempt<'_>> {
        self.reserve_at(key, Instant::now())
    }

    fn reserve_at(&self, key: ThrottleKey, now: Instant) -> Option<ClientAuthAttempt<'_>> {
        self.failures
            .has_budget_at(&key, now)
            .then_some(ClientAuthAttempt {
                throttle: self,
                key,
            })
    }

    #[cfg(test)]
    fn tracked(&self) -> usize {
        self.failures.tracked()
    }
}

#[must_use]
#[derive(Debug)]
pub struct ClientAuthAttempt<'a> {
    throttle: &'a ClientAuthThrottle,
    key: ThrottleKey,
}

impl ClientAuthAttempt<'_> {
    pub fn failed(self) {
        self.failed_at(Instant::now());
    }

    fn failed_at(self, now: Instant) {
        self.throttle.failures.spend_at(self.key, now);
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

    fn key(ip: IpAddr, realm: &str, client_id: &str) -> ThrottleKey {
        ThrottleKey::new(ip, realm, client_id)
    }

    #[test]
    fn requests_that_do_not_fail_leave_no_trace() {
        let throttle = ClientAuthThrottle::per_minute(3);
        let now = Instant::now();

        for host in 1..=50u8 {
            let attempt = throttle
                .reserve_at(key(ip(host), "master", "console"), now)
                .expect("a fresh address may try");
            drop(attempt);
        }

        assert_eq!(throttle.tracked(), 0);
    }

    #[test]
    fn requests_in_flight_never_exhaust_the_budget() {
        let throttle = ClientAuthThrottle::per_minute(3);
        let now = Instant::now();

        let in_flight: Vec<_> = (0..50)
            .map(|_| {
                throttle
                    .reserve_at(key(ip(10), "master", "console"), now)
                    .expect("requests that have not failed are not throttled")
            })
            .collect();

        assert_eq!(in_flight.len(), 50);
        assert_eq!(throttle.tracked(), 0);
    }

    #[test]
    fn failed_attempts_block_the_client_until_the_window_refills() {
        let throttle = ClientAuthThrottle::per_minute(3);
        let now = Instant::now();
        let blocked = key(ip(8), "master", "console");

        for _ in 0..3 {
            throttle
                .reserve_at(blocked.clone(), now)
                .expect("under the budget")
                .failed_at(now);
        }

        assert!(throttle.reserve_at(blocked.clone(), now).is_none());
        assert!(
            throttle
                .reserve_at(blocked.clone(), now + Duration::from_secs(10))
                .is_none()
        );
        assert!(
            throttle
                .reserve_at(blocked, now + Duration::from_secs(21))
                .is_some()
        );
        assert_eq!(throttle.tracked(), 1, "only keys with failures are tracked");
    }

    #[test]
    fn failures_of_one_client_do_not_throttle_another_client_behind_the_same_address() {
        let throttle = ClientAuthThrottle::per_minute(2);
        let now = Instant::now();
        let shared = ip(20);

        for _ in 0..2 {
            throttle
                .reserve_at(key(shared, "master", "guessed"), now)
                .expect("under the budget")
                .failed_at(now);
        }

        assert!(
            throttle
                .reserve_at(key(shared, "master", "guessed"), now)
                .is_none()
        );
        assert!(
            throttle
                .reserve_at(key(shared, "master", "console"), now)
                .is_some()
        );
        assert!(
            throttle
                .reserve_at(key(shared, "other", "guessed"), now)
                .is_some()
        );
        assert!(
            throttle
                .reserve_at(key(ip(21), "master", "guessed"), now)
                .is_some()
        );
    }

    #[test]
    fn a_blocked_key_is_not_evicted_to_make_room_for_new_keys() {
        let limiter = KeyedRateLimiter::with_limits(2, 3);
        let now = Instant::now();

        assert!(limiter.spend_at("blocked", now));
        assert!(limiter.spend_at("blocked", now));
        assert!(!limiter.has_budget_at(&"blocked", now));

        for (second, name) in ["a", "b", "c", "d", "e"].into_iter().enumerate() {
            limiter.spend_at(name, now + Duration::from_secs(second as u64 + 1));
        }

        let later = now + Duration::from_secs(6);
        assert!(!limiter.has_budget_at(&"blocked", later));
        assert!(!limiter.spend_at("blocked", later));
        assert!(limiter.tracked() <= 3);
    }

    #[test]
    fn a_new_key_is_admitted_untracked_while_every_tracked_key_is_blocked() {
        let limiter = KeyedRateLimiter::with_limits(1, 2);
        let now = Instant::now();

        assert!(limiter.spend_at("a", now));
        assert!(limiter.spend_at("b", now));

        assert!(limiter.has_budget_at(&"c", now));
        assert!(limiter.spend_at("c", now));
        assert_eq!(limiter.tracked(), 2);
        assert!(!limiter.has_budget_at(&"a", now));
        assert!(!limiter.has_budget_at(&"b", now));
    }

    #[test]
    fn blocked_identities_never_deny_admission_to_independent_clients() {
        let throttle = ClientAuthThrottle::with_limits(1, 3);
        let now = Instant::now();
        let junk = |n: usize| key(ip(40), "master", &format!("junk-{n}"));

        for n in 0..3 {
            throttle
                .reserve_at(junk(n), now)
                .expect("a fresh key may try")
                .failed_at(now);
        }
        assert_eq!(throttle.tracked(), 3);

        let independent = key(ip(41), "master", "console");
        throttle
            .reserve_at(independent.clone(), now)
            .expect("an independent client is not denied by someone else's failures")
            .failed_at(now);

        assert_eq!(throttle.tracked(), 3);
        for n in 0..3 {
            assert!(throttle.reserve_at(junk(n), now).is_none());
        }
        assert!(throttle.reserve_at(independent, now).is_some());
    }

    #[test]
    fn a_new_key_is_accepted_once_a_tracked_key_has_a_token_again() {
        let limiter = KeyedRateLimiter::with_limits(1, 2);
        let now = Instant::now();

        assert!(limiter.spend_at("a", now));
        assert!(limiter.spend_at("b", now + Duration::from_secs(30)));

        let later = now + Duration::from_secs(61);
        assert!(limiter.has_budget_at(&"c", later));
        assert!(limiter.spend_at("c", later));
        assert_eq!(limiter.tracked(), 2);
    }

    #[test]
    fn client_ids_sharing_a_long_prefix_keep_separate_budgets() {
        let throttle = ClientAuthThrottle::per_minute(1);
        let now = Instant::now();
        let prefix = "a".repeat(200);
        let guessed = key(ip(30), "master", &format!("{prefix}x"));
        let other = key(ip(30), "master", &format!("{prefix}y"));

        assert_ne!(guessed, other);
        throttle
            .reserve_at(guessed.clone(), now)
            .expect("under the budget")
            .failed_at(now);

        assert!(throttle.reserve_at(guessed, now).is_none());
        assert!(throttle.reserve_at(other, now).is_some());
    }

    #[test]
    fn the_same_identifiers_always_give_the_same_key() {
        let long = "a".repeat(10_000);

        assert_eq!(
            key(ip(31), "master", &long),
            key(ip(31), "master", &long.clone())
        );
    }

    #[test]
    fn the_boundary_between_realm_and_client_id_is_part_of_the_key() {
        assert_ne!(key(ip(32), "ab", "c"), key(ip(32), "a", "bc"));
    }
}

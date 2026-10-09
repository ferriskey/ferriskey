use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const DEFAULT_REQUESTS_PER_MINUTE: u32 = 10;
pub const MAX_TRACKED_ADDRESSES: usize = 10_000;

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
}

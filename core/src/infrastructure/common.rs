use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Tells a unique-constraint violation apart from a genuine server fault.
///
/// A schema-level uniqueness conflict is a caller asking for a value that is
/// taken, not a failure of the server. Matching on the SQLSTATE rather than the
/// message keeps this independent of the constraint's name and of the server's
/// locale.
pub fn is_unique_violation(error: &sea_orm::DbErr) -> bool {
    matches!(
        error.sql_err(),
        Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
    )
}

#[derive(Debug)]
pub struct TtlCache<K, V> {
    entries: Mutex<HashMap<K, (V, Instant)>>,
    ttl: Duration,
}

impl<K: Eq + Hash, V: Clone> TtlCache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let entries = self.entries.lock().ok()?;
        entries
            .get(key)
            .filter(|(_, loaded_at)| loaded_at.elapsed() < self.ttl)
            .map(|(value, _)| value.clone())
    }

    pub fn insert(&self, key: K, value: V) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(key, (value, Instant::now()));
        }
    }

    pub fn remove<Q>(&self, key: &Q)
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Ok(mut entries) = self.entries.lock() {
            entries.remove(key);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }
}

//! Tamper-evident hash chaining for SeaWatch audit events.
//!
//! # Preimage format
//!
//! ```text
//! {id}|{realm_id}|{actor_id}|{actor_type}|{event_type}|{status}|
//! {target_type}|{target_id}|{resource}|{timestamp}|{trace_id}|
//! {ip_address}|{user_agent}|{details}
//! ```
//!
//! Fields use their `Display` representation. Optional fields that are `None`
//! are encoded as the literal string `NONE`. The `details` JSON value is
//! serialised with [`serde_json::to_string`] (compact, deterministic for the
//! same value) when present.
//!
//! The 32-byte `prev_hash` is appended as raw bytes after the UTF-8 encoded
//! preimage text, with no separator.
//!
//! The genesis event uses `prev_hash = [0u8; 32]`.
//!
//! # Chain scope
//!
//! Chains are per-realm. Events from different realms form independent chains.

use sha2::{Digest, Sha256};

use super::entities::SecurityEvent;

/// 32 zero bytes used as the genesis `prev_hash`.
pub const GENESIS_PREV_HASH: [u8; 32] = [0u8; 32];

/// Compute `SHA-256(canonical_preimage(event) || prev_hash_bytes)`.
///
/// This is a **pure** function — no I/O, trivially unit-testable.
pub fn compute_event_hash(event: &SecurityEvent, prev_hash: &[u8; 32]) -> [u8; 32] {
    let preimage = build_preimage(event);
    let mut hasher = Sha256::new();
    hasher.update(preimage.as_bytes());
    hasher.update(prev_hash);
    hasher.finalize().into()
}

/// Result of a chain verification pass.
#[derive(Debug, PartialEq)]
pub enum VerifyResult {
    /// All events verify correctly.
    Ok,
    /// A hash mismatch was detected at the given 0-based index.
    Tampered { index: usize },
    /// The `prev_hash` link is broken at the given index (deletion / reorder).
    BrokenLink { index: usize },
}

/// Verify a slice of [`SecurityEvent`]s that form a contiguous per-realm chain.
///
/// Events must be ordered by insertion order (ascending). The first event's
/// `prev_hash` must be [`GENESIS_PREV_HASH`] or the caller-supplied `expected_genesis`.
///
/// Returns [`VerifyResult::Ok`] when all hashes are consistent.
pub fn verify_chain(events: &[SecurityEvent]) -> VerifyResult {
    verify_chain_from(events, &GENESIS_PREV_HASH)
}

/// Like [`verify_chain`] but allows supplying a custom genesis hash (useful for
/// verifying sub-chains that start after a known anchor).
pub fn verify_chain_from(events: &[SecurityEvent], genesis: &[u8; 32]) -> VerifyResult {
    let mut expected_prev = *genesis;

    for (i, event) in events.iter().enumerate() {
        let stored_prev = match event.prev_hash {
            Some(h) => h,
            None => return VerifyResult::Tampered { index: i },
        };
        let stored_hash = match event.event_hash {
            Some(h) => h,
            None => return VerifyResult::Tampered { index: i },
        };

        if stored_prev != expected_prev {
            return VerifyResult::BrokenLink { index: i };
        }

        let recomputed = compute_event_hash(event, &stored_prev);
        if recomputed != stored_hash {
            return VerifyResult::Tampered { index: i };
        }

        expected_prev = stored_hash;
    }

    VerifyResult::Ok
}

/// Export a chain as JSONL (one JSON object per line).
///
/// Each line includes all event fields plus `event_hash` and `prev_hash` as
/// lowercase hex strings. This provides a minimal WORM-style append-only export.
pub fn export_chain_jsonl(events: &[SecurityEvent]) -> String {
    use serde_json::json;

    let lines: Vec<String> = events
        .iter()
        .map(|e| {
            let id: uuid::Uuid = e.id.0;
            let realm_id: uuid::Uuid = e.realm_id.into();
            let obj = json!({
                "id": id,
                "realm_id": realm_id,
                "actor_id": e.actor_id,
                "actor_type": e.actor_type.as_ref().map(|t| t.to_string()),
                "event_type": e.event_type.to_string(),
                "status": e.status.to_string(),
                "target_type": e.target_type,
                "target_id": e.target_id,
                "resource": e.resource,
                "timestamp": e.timestamp.to_rfc3339(),
                "trace_id": e.trace_id,
                "ip_address": e.ip_address,
                "user_agent": e.user_agent,
                "details": e.details,
                "prev_hash": e.prev_hash.map(hex::encode),
                "event_hash": e.event_hash.map(hex::encode),
            });
            serde_json::to_string(&obj).unwrap_or_default()
        })
        .collect();
    lines.join("\n")
}

fn build_preimage(event: &SecurityEvent) -> String {
    let id: uuid::Uuid = event.id.0;
    let id = id.to_string();
    let realm_uuid: uuid::Uuid = event.realm_id.into();
    let realm_id = realm_uuid.to_string();
    let actor_id = opt_str(event.actor_id.map(|u| u.to_string()));
    let actor_type = opt_str(event.actor_type.as_ref().map(|t| t.to_string()));
    let event_type = event.event_type.to_string();
    let status = event.status.to_string();
    let target_type = opt_str(event.target_type.clone());
    let target_id = opt_str(event.target_id.map(|u| u.to_string()));
    let resource = opt_str(event.resource.clone());
    let timestamp = event
        .timestamp
        .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    let trace_id = opt_str(event.trace_id.clone());
    let ip_address = opt_str(event.ip_address.clone());
    let user_agent = opt_str(event.user_agent.clone());
    let details = opt_str(
        event
            .details
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok()),
    );

    format!(
        "{id}|{realm_id}|{actor_id}|{actor_type}|{event_type}|{status}|\
         {target_type}|{target_id}|{resource}|{timestamp}|{trace_id}|\
         {ip_address}|{user_agent}|{details}"
    )
}

fn opt_str(v: Option<String>) -> String {
    v.unwrap_or_else(|| "NONE".to_string())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::realm::entities::RealmId;
    use crate::domain::seawatch::entities::{
        EventStatus, SecurityEvent, SecurityEventId, SecurityEventType,
    };

    use super::*;

    fn make_event(realm_id: RealmId) -> SecurityEvent {
        SecurityEvent {
            id: SecurityEventId::new(),
            realm_id,
            actor_id: Some(Uuid::new_v4()),
            actor_type: None,
            event_type: SecurityEventType::LoginSuccess,
            status: EventStatus::Success,
            target_type: None,
            target_id: None,
            resource: None,
            timestamp: Utc::now(),
            trace_id: None,
            ip_address: None,
            user_agent: None,
            details: None,
            event_hash: None,
            prev_hash: None,
        }
    }

    fn chain_events(count: usize) -> Vec<SecurityEvent> {
        let realm_id = RealmId::new(Uuid::new_v4());
        let mut prev = GENESIS_PREV_HASH;
        let mut events = Vec::with_capacity(count);
        for _ in 0..count {
            let mut e = make_event(realm_id.clone());
            let hash = compute_event_hash(&e, &prev);
            e.prev_hash = Some(prev);
            e.event_hash = Some(hash);
            prev = hash;
            events.push(e);
        }
        events
    }

    #[test]
    fn chain_of_n_events_verifies_ok() {
        let events = chain_events(5);
        assert_eq!(verify_chain(&events), VerifyResult::Ok);
    }

    #[test]
    fn empty_chain_verifies_ok() {
        assert_eq!(verify_chain(&[]), VerifyResult::Ok);
    }

    #[test]
    fn single_genesis_event_verifies_ok() {
        let events = chain_events(1);
        assert_eq!(verify_chain(&events), VerifyResult::Ok);
    }

    #[test]
    fn genesis_hash_is_deterministic() {
        let realm_id = RealmId::new(Uuid::new_v4());
        let mut e = make_event(realm_id);
        // Fix timestamp so the preimage is stable across calls.
        e.timestamp = chrono::DateTime::from_timestamp_nanos(1_000_000_000);
        let h1 = compute_event_hash(&e, &GENESIS_PREV_HASH);
        let h2 = compute_event_hash(&e, &GENESIS_PREV_HASH);
        assert_eq!(h1, h2);
    }

    #[test]
    fn mutating_event_field_breaks_chain_at_that_index() {
        let mut events = chain_events(4);
        // Tamper: change the event_type on index 1 without recomputing hashes.
        events[1].event_type = SecurityEventType::LoginFailure;
        assert_eq!(verify_chain(&events), VerifyResult::Tampered { index: 1 });
    }

    #[test]
    fn removing_event_breaks_link() {
        let mut events = chain_events(4);
        // Remove event at index 1; event[1] (old index 2) now has a prev_hash
        // pointing to old event[1] which is gone.
        events.remove(1);
        assert_eq!(verify_chain(&events), VerifyResult::BrokenLink { index: 1 });
    }

    #[test]
    fn reordering_events_breaks_chain() {
        let mut events = chain_events(3);
        events.swap(0, 1);
        // Event[0] (originally index 1) has prev_hash = event[0]'s hash, but
        // expected genesis prev is [0u8;32].
        assert!(verify_chain(&events) != VerifyResult::Ok);
    }

    #[test]
    fn missing_event_hash_is_detected_as_tamper() {
        let mut events = chain_events(3);
        events[1].event_hash = None;
        assert_eq!(verify_chain(&events), VerifyResult::Tampered { index: 1 });
    }

    #[test]
    fn export_jsonl_produces_one_line_per_event() {
        let events = chain_events(3);
        let output = export_chain_jsonl(&events);
        assert_eq!(output.lines().count(), 3);
        // Each line must be valid JSON containing event_hash.
        for line in output.lines() {
            let v: serde_json::Value = serde_json::from_str(line).expect("valid JSON");
            assert!(v.get("event_hash").is_some());
            assert!(v.get("prev_hash").is_some());
        }
    }
}

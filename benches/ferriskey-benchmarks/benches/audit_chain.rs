use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ferriskey_domain::realm::RealmId;
use ferriskey_seawatch::entities::{EventStatus, SecurityEvent, SecurityEventType};
use ferriskey_seawatch::hashing::{GENESIS_PREV_HASH, compute_event_hash, verify_chain};
use uuid::Uuid;

fn chain_of(count: usize) -> Vec<SecurityEvent> {
    let realm_id = RealmId::new(Uuid::new_v4());
    let user_id = Uuid::new_v4();
    let mut prev = GENESIS_PREV_HASH;
    (0..count)
        .map(|_| {
            let mut event = SecurityEvent::new(
                realm_id,
                SecurityEventType::LoginSuccess,
                EventStatus::Success,
                user_id,
            );
            let hash = compute_event_hash(&event, &prev);
            event.prev_hash = Some(prev);
            event.event_hash = Some(hash);
            prev = hash;
            event
        })
        .collect()
}

fn audit_chain(c: &mut Criterion) {
    let events = chain_of(1_000);
    let last = events.last().expect("a thousand events");

    c.bench_function("hash one audit event", |b| {
        b.iter(|| compute_event_hash(black_box(last), &GENESIS_PREV_HASH))
    });

    c.bench_function("verify a chain of 1000 audit events", |b| {
        b.iter(|| verify_chain(black_box(&events)))
    });
}

criterion_group!(benches, audit_chain);
criterion_main!(benches);

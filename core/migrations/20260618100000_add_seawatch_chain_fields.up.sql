-- Add tamper-evident hash-chain fields to security_events.
-- event_hash = SHA-256(canonical_preimage || prev_hash_bytes)
-- prev_hash  = event_hash of the previous event in the realm's chain;
--              genesis event uses 32 zero bytes.
ALTER TABLE security_events
    ADD COLUMN IF NOT EXISTS event_hash TEXT,
    ADD COLUMN IF NOT EXISTS prev_hash  TEXT;

-- Index to efficiently locate the current chain head per realm.
CREATE INDEX IF NOT EXISTS idx_security_events_realm_created
    ON security_events (realm_id, created_at DESC);

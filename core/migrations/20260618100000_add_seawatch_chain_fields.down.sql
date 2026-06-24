DROP INDEX IF EXISTS idx_security_events_realm_created;

ALTER TABLE security_events
    DROP COLUMN IF EXISTS event_hash,
    DROP COLUMN IF EXISTS prev_hash;

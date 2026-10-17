CREATE TABLE IF NOT EXISTS security_event_chain_heads (
    realm_id   UUID PRIMARY KEY REFERENCES realms(id) ON DELETE CASCADE,
    event_hash TEXT,
    prev_hash  TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO security_event_chain_heads (realm_id, event_hash, prev_hash)
SELECT DISTINCT ON (realm_id) realm_id, event_hash, prev_hash
FROM security_events
ORDER BY realm_id, created_at DESC, id DESC
ON CONFLICT (realm_id) DO NOTHING;

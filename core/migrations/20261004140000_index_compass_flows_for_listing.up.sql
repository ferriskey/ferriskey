CREATE INDEX IF NOT EXISTS idx_compass_flows_realm_created
    ON compass_flows (realm_id, created_at DESC, id DESC);

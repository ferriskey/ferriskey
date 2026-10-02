CREATE INDEX IF NOT EXISTS idx_compass_flows_started_at
    ON compass_flows (started_at);

CREATE INDEX IF NOT EXISTS idx_compass_flows_pending_started_at
    ON compass_flows (started_at)
    WHERE status = 'pending';

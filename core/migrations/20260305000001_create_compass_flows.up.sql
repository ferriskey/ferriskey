CREATE TABLE compass_flows (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    client_id UUID,
    user_id UUID,
    grant_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    ip_address VARCHAR(45),
    user_agent TEXT,
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    duration_ms BIGINT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_compass_flows_realm_started ON compass_flows(realm_id, started_at DESC);
CREATE INDEX idx_compass_flows_status ON compass_flows(realm_id, status);
CREATE INDEX idx_compass_flows_client ON compass_flows(client_id);
CREATE INDEX idx_compass_flows_user ON compass_flows(user_id);

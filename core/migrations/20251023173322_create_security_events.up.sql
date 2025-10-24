-- Add up migration script here
CREATE TABLE security_events (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL,
    actor_id UUID,
    actor_type VARCHAR(50),
    event_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    target_type VARCHAR(50),
    target_id UUID,
    resource TEXT,
    timestamp TIMESTAMPTZ NOT NULL,
    trace_id VARCHAR(255),
    ip_address VARCHAR(45),
    user_agent TEXT,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_security_events_realm
        FOREIGN KEY (realm_id) 
        REFERENCES realms(id) 
        ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX idx_security_events_realm_timestamp ON security_events(realm_id, timestamp DESC);
CREATE INDEX idx_security_events_actor_id ON security_events(actor_id);
CREATE INDEX idx_security_events_event_type ON security_events(event_type);
CREATE INDEX idx_security_events_status ON security_events(status);
CREATE INDEX idx_security_events_target_id ON security_events(target_id);
CREATE INDEX idx_security_events_trace_id ON security_events(trace_id);
CREATE INDEX idx_security_events_ip_address ON security_events(ip_address);
CREATE INDEX idx_security_events_realm_actor_timestamp ON security_events(realm_id, actor_id, timestamp DESC);
CREATE INDEX idx_security_events_realm_type_timestamp ON security_events(realm_id, event_type, timestamp DESC);
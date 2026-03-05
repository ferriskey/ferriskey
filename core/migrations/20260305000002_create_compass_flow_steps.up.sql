CREATE TABLE compass_flow_steps (
    id UUID PRIMARY KEY,
    flow_id UUID NOT NULL REFERENCES compass_flows(id) ON DELETE CASCADE,
    step_name VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL,
    duration_ms BIGINT,
    error_code VARCHAR(100),
    error_message TEXT,
    started_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_compass_steps_flow ON compass_flow_steps(flow_id);

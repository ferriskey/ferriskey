ALTER TABLE auth_sessions ADD COLUMN compass_flow_id UUID REFERENCES compass_flows(id) ON DELETE SET NULL;

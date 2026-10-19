-- no-transaction
ALTER TABLE auth_sessions
  ADD CONSTRAINT auth_sessions_compass_flow_id_fkey
  FOREIGN KEY (compass_flow_id) REFERENCES compass_flows(id) ON DELETE SET NULL NOT VALID;
UPDATE auth_sessions SET compass_flow_id = NULL
WHERE compass_flow_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM compass_flows WHERE compass_flows.id = auth_sessions.compass_flow_id);
ALTER TABLE auth_sessions VALIDATE CONSTRAINT auth_sessions_compass_flow_id_fkey;

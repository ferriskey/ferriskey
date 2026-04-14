DROP TABLE IF EXISTS realm_maintenance_whitelist;
DROP TABLE IF EXISTS client_maintenance_whitelist;
ALTER TABLE clients DROP COLUMN IF EXISTS maintenance_session_strategy;
ALTER TABLE clients DROP COLUMN IF EXISTS maintenance_reason;
ALTER TABLE clients DROP COLUMN IF EXISTS maintenance_enabled;

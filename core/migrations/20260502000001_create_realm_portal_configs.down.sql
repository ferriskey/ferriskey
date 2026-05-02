-- Add down migration script here
DROP INDEX IF EXISTS idx_realm_portal_configs_realm_id;
DROP TABLE IF EXISTS realm_portal_configs;

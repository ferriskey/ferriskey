-- Add down migration script here
ALTER TABLE realm_settings DROP CONSTRAINT fk_realm_settings_realm_id;
DROP TABLE realm_settings;

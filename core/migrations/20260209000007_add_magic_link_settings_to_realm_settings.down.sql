DROP INDEX IF EXISTS idx_realm_settings_magic_link_enabled;
ALTER TABLE realm_settings
DROP COLUMN IF EXISTS magic_link_ttl_minutes,
DROP COLUMN IF EXISTS magic_link_enabled;

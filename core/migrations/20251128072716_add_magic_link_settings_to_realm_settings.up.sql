-- Add magic link settings to realm_settings table
ALTER TABLE realm_settings
ADD COLUMN magic_link_enabled BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN magic_link_ttl_minutes INTEGER NOT NULL DEFAULT 60;

CREATE INDEX idx_realm_settings_magic_link_enabled ON realm_settings(magic_link_enabled)
WHERE magic_link_enabled = TRUE;

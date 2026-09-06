-- Add up migration script here

ALTER TABLE realm_settings
    ADD COLUMN IF NOT EXISTS edit_username_enabled BOOLEAN NOT NULL DEFAULT false;

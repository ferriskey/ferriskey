-- Add down migration script here

ALTER TABLE realm_settings
    DROP COLUMN IF EXISTS edit_username_enabled;

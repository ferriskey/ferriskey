-- Add down migration script here

ALTER TABLE realm_settings
    DROP COLUMN IF EXISTS require_mfa;

ALTER TABLE roles
    DROP COLUMN IF EXISTS require_mfa;

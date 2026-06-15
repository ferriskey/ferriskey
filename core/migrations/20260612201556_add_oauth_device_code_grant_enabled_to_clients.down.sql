-- Add down migration script here

ALTER TABLE clients
    DROP COLUMN IF EXISTS oauth_device_code_grant_enabled;

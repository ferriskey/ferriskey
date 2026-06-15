-- Add up migration script here

ALTER TABLE clients
    ADD COLUMN IF NOT EXISTS oauth_device_code_grant_enabled BOOLEAN DEFAULT false;

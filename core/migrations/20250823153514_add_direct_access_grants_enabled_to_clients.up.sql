-- Add up migration script here

ALTER TABLE clients
    ADD COLUMN IF NOT EXISTS direct_access_grants_enabled BOOLEAN DEFAULT false;

-- Add down migration script here

ALTER TABLE clients
    DROP COLUMN IF EXISTS direct_access_grants_enabled;

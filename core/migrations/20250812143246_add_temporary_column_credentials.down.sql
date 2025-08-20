-- Add down migration script here
ALTER TABLE credentials
    DROP COLUMN IF EXISTS temporary;

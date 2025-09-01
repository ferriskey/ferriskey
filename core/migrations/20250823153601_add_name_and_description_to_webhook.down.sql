-- Add down migration script here

ALTER TABLE webhooks
    DROP COLUMN IF NOT EXISTS name;
ALTER TABLE webhooks
    DROP COLUMN IF NOT EXISTS description;

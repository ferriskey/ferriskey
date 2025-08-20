-- Add up migration script here

ALTER TABLE credentials
    ADD COLUMN IF NOT EXISTS temporary BOOLEAN DEFAULT FALSE;

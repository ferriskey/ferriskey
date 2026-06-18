-- Add up migration script here

ALTER TABLE realm_settings
    ADD COLUMN IF NOT EXISTS lockout_threshold INT NOT NULL DEFAULT 10,
    ADD COLUMN IF NOT EXISTS lockout_duration_seconds INT NOT NULL DEFAULT 900;

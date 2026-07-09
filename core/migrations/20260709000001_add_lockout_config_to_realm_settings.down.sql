-- Add down migration script here

ALTER TABLE realm_settings
    DROP COLUMN IF EXISTS lockout_threshold,
    DROP COLUMN IF EXISTS lockout_duration_seconds;

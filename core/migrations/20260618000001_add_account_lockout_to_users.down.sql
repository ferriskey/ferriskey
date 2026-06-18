-- Add down migration script here

ALTER TABLE users
    DROP COLUMN IF EXISTS failed_login_attempts,
    DROP COLUMN IF EXISTS locked_until;

-- Add down migration script here

DROP INDEX IF EXISTS idx_refresh_tokens_status;
DROP INDEX IF EXISTS idx_refresh_tokens_family_id;

ALTER TABLE refresh_tokens
    DROP COLUMN IF EXISTS rotated_at,
    DROP COLUMN IF EXISTS replaced_by,
    DROP COLUMN IF EXISTS status,
    DROP COLUMN IF EXISTS family_id;

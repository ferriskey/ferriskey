-- Add up migration script here

-- Refresh token rotation: add family lineage and status lifecycle
ALTER TABLE refresh_tokens
    ADD COLUMN IF NOT EXISTS family_id   UUID        NOT NULL DEFAULT gen_random_uuid(),
    ADD COLUMN IF NOT EXISTS status      TEXT        NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS replaced_by UUID        NULL,
    ADD COLUMN IF NOT EXISTS rotated_at  TIMESTAMPTZ NULL;

-- Backfill: tokens that were already revoked via the old boolean flag
UPDATE refresh_tokens
SET status = 'revoked'
WHERE revoked = true AND status = 'active';

-- Index for fast family revocation
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_family_id ON refresh_tokens (family_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_status    ON refresh_tokens (status);

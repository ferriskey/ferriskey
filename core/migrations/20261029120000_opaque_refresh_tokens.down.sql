DROP INDEX IF EXISTS idx_refresh_tokens_token_hash;

ALTER TABLE refresh_tokens
    DROP COLUMN IF EXISTS scope,
    DROP COLUMN IF EXISTS client_id,
    DROP COLUMN IF EXISTS token_hash;

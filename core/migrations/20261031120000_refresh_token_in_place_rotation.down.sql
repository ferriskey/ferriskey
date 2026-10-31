DROP INDEX IF EXISTS idx_refresh_tokens_previous_token_hash;
DROP INDEX IF EXISTS idx_refresh_tokens_previous_jti;

ALTER TABLE refresh_tokens
    DROP COLUMN IF EXISTS previous_token_hash,
    DROP COLUMN IF EXISTS previous_jti;

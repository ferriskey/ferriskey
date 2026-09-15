DROP INDEX IF EXISTS idx_user_sessions_sso_token_hash;

ALTER TABLE user_sessions
    DROP COLUMN IF EXISTS sso_token_hash;

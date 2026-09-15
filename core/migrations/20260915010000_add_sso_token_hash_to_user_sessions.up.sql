ALTER TABLE user_sessions
    ADD COLUMN IF NOT EXISTS sso_token_hash VARCHAR(64);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_sessions_sso_token_hash
    ON user_sessions (sso_token_hash);

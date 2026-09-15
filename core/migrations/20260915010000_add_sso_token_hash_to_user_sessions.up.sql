-- The browser's credential for its SSO session.
--
-- The cookie cannot simply carry `user_sessions.id`: that id is handed out by the
-- session listing API to anyone holding `view_users`, which would turn a read-only
-- permission into impersonation. The cookie carries a secret instead, and only its
-- SHA-256 is kept here.
ALTER TABLE user_sessions
    ADD COLUMN IF NOT EXISTS sso_token_hash VARCHAR(64);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_sessions_sso_token_hash
    ON user_sessions (sso_token_hash);

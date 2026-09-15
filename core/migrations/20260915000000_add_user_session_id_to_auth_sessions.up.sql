ALTER TABLE auth_sessions
    ADD COLUMN IF NOT EXISTS user_session_id UUID
    REFERENCES user_sessions(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_auth_sessions_user_session_id
    ON auth_sessions (user_session_id);

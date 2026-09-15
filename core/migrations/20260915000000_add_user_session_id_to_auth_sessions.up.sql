-- The SSO session a login belongs to, carried from the interactive login step
-- through to the token exchange so the exchange binds its `sid` to the session
-- that authenticated the user instead of opening a new one per application.
--
-- ON DELETE SET NULL: revoking a session hard-deletes its row, and an auth
-- session left pointing at it must simply lose the link rather than disappear.
ALTER TABLE auth_sessions
    ADD COLUMN IF NOT EXISTS user_session_id UUID
    REFERENCES user_sessions(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_auth_sessions_user_session_id
    ON auth_sessions (user_session_id);

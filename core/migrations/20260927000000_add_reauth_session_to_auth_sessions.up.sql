-- The live SSO session a browser still held when `/auth` sent it to the login
-- page anyway (prompt=login, max_age exceeded, a step still due). When the
-- login completes for the same user, it re-authenticates into that session
-- instead of opening a second one. SET NULL: a revoked session simply stops
-- being a candidate.
ALTER TABLE auth_sessions
    ADD COLUMN IF NOT EXISTS reauth_session_id UUID
    REFERENCES user_sessions(id) ON DELETE SET NULL;

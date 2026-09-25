-- An auth session bound to a user session holds an authorization code minted
-- against it. With SET NULL, revoking the session left that code looking
-- unbound, and exchanging it opened a fresh session. The code must die with
-- the session instead.
ALTER TABLE auth_sessions
    DROP CONSTRAINT IF EXISTS auth_sessions_user_session_id_fkey,
    ADD CONSTRAINT auth_sessions_user_session_id_fkey
        FOREIGN KEY (user_session_id) REFERENCES user_sessions(id) ON DELETE CASCADE;

ALTER TABLE auth_sessions
    DROP CONSTRAINT IF EXISTS auth_sessions_user_session_id_fkey,
    ADD CONSTRAINT auth_sessions_user_session_id_fkey
        FOREIGN KEY (user_session_id) REFERENCES user_sessions(id) ON DELETE SET NULL;

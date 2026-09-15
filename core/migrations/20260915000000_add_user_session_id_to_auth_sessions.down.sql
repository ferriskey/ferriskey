DROP INDEX IF EXISTS idx_auth_sessions_user_session_id;

ALTER TABLE auth_sessions
    DROP COLUMN IF EXISTS user_session_id;

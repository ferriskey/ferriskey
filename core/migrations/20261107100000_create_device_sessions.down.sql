DROP INDEX IF EXISTS idx_user_sessions_device_session_id;

ALTER TABLE user_sessions
    DROP COLUMN IF EXISTS device_session_id;

DROP TABLE IF EXISTS device_sessions;

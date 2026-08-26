DROP INDEX IF EXISTS idx_auth_sessions_protocol;

DELETE FROM auth_sessions WHERE response_type IS NULL OR scope IS NULL;

ALTER TABLE auth_sessions
    ALTER COLUMN response_type SET NOT NULL;

ALTER TABLE auth_sessions
    ALTER COLUMN scope SET NOT NULL;

ALTER TABLE auth_sessions
    DROP CONSTRAINT IF EXISTS auth_sessions_protocol_check;

ALTER TABLE auth_sessions
    DROP COLUMN IF EXISTS protocol;

ALTER TABLE auth_sessions
ALTER COLUMN redirect_uri TYPE TEXT;

ALTER TABLE auth_sessions
ALTER COLUMN response_type TYPE TEXT;

ALTER TABLE auth_sessions
ALTER COLUMN scope TYPE TEXT;

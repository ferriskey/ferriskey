DELETE FROM auth_sessions
WHERE length(redirect_uri) > 255
   OR length(response_type) > 255
   OR length(scope) > 255;

ALTER TABLE auth_sessions
ALTER COLUMN redirect_uri TYPE VARCHAR(255);

ALTER TABLE auth_sessions
ALTER COLUMN response_type TYPE VARCHAR(255);

ALTER TABLE auth_sessions
ALTER COLUMN scope TYPE VARCHAR(255);

ALTER TABLE auth_sessions
    ADD COLUMN IF NOT EXISTS protocol VARCHAR(32) NOT NULL DEFAULT 'openid-connect';

UPDATE auth_sessions SET protocol = 'openid-connect' WHERE protocol IS NULL;

ALTER TABLE auth_sessions
    DROP CONSTRAINT IF EXISTS auth_sessions_protocol_check;

ALTER TABLE auth_sessions
    ADD CONSTRAINT auth_sessions_protocol_check
    CHECK (protocol IN ('openid-connect', 'saml'));

ALTER TABLE auth_sessions
    ALTER COLUMN response_type DROP NOT NULL;

ALTER TABLE auth_sessions
    ALTER COLUMN scope DROP NOT NULL;

CREATE INDEX IF NOT EXISTS idx_auth_sessions_protocol ON auth_sessions (protocol);

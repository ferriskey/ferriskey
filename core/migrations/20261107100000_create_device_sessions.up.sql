CREATE TABLE device_sessions (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    device_token_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX idx_device_sessions_token_hash ON device_sessions (device_token_hash);
CREATE INDEX idx_device_sessions_realm_id ON device_sessions (realm_id);
CREATE INDEX idx_device_sessions_expires_at ON device_sessions (expires_at);

ALTER TABLE user_sessions
    ADD COLUMN device_session_id UUID REFERENCES device_sessions(id) ON DELETE SET NULL;

CREATE INDEX idx_user_sessions_device_session_id ON user_sessions (device_session_id);

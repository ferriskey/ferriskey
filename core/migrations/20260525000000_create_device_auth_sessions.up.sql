CREATE TABLE device_auth_sessions (
    device_code        UUID PRIMARY KEY,
    user_code          VARCHAR(9) NOT NULL UNIQUE,
    client_id          UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    realm_id           UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    user_id            UUID NULL REFERENCES users(id) ON DELETE CASCADE,
    scope              TEXT NULL,
    status             VARCHAR(32) NOT NULL DEFAULT 'pending',
    interval_seconds   INTEGER NOT NULL DEFAULT 5,
    expires_at         TIMESTAMPTZ NOT NULL,
    last_polled_at     TIMESTAMPTZ NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_device_auth_sessions_user_code ON device_auth_sessions(user_code);
CREATE INDEX idx_device_auth_sessions_client_status ON device_auth_sessions(client_id, status);
CREATE INDEX idx_device_auth_sessions_expires_at ON device_auth_sessions(expires_at);

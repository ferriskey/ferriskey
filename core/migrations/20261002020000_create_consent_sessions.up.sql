CREATE TABLE consent_sessions (
    id             UUID PRIMARY KEY,
    realm_id       UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id      UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    granted_scopes TEXT[] NOT NULL DEFAULT '{}',
    denied_scopes  TEXT[] NOT NULL DEFAULT '{}',
    expires_at     TIMESTAMP NOT NULL,
    created_at     TIMESTAMP NOT NULL DEFAULT now(),
    updated_at     TIMESTAMP NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_consent_sessions_realm_user_client
    ON consent_sessions (realm_id, user_id, client_id);

CREATE INDEX idx_consent_sessions_expires_at
    ON consent_sessions (expires_at);

CREATE TABLE consent_sessions (
    id         UUID PRIMARY KEY,
    realm_id   UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id  UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_consent_sessions_realm_user_client
    ON consent_sessions (realm_id, user_id, client_id);

CREATE INDEX idx_consent_sessions_expires_at
    ON consent_sessions (expires_at);

CREATE TABLE consent_session_scopes (
    consent_session_id UUID NOT NULL REFERENCES consent_sessions(id) ON DELETE CASCADE,
    scope              TEXT NOT NULL,
    granted            BOOLEAN NOT NULL,
    PRIMARY KEY (consent_session_id, scope)
);

CREATE INDEX idx_consent_session_scopes_scope
    ON consent_session_scopes (scope);

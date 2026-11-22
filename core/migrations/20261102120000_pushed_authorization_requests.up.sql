ALTER TABLE clients
    ADD COLUMN require_pushed_authorization_requests BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE realm_settings
    ADD COLUMN require_pushed_authorization_requests BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE pushed_authorization_requests (
    request_uri_hash TEXT PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    parameters JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_pushed_authorization_requests_expires_at
    ON pushed_authorization_requests (expires_at);

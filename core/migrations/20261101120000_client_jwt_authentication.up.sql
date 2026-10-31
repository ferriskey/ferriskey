ALTER TABLE clients
    ADD COLUMN token_endpoint_auth_method TEXT NOT NULL DEFAULT 'client_secret_basic',
    ADD COLUMN jwks JSONB NULL,
    ADD COLUMN jwks_uri TEXT NULL,
    ADD CONSTRAINT clients_single_key_source CHECK (jwks IS NULL OR jwks_uri IS NULL);

UPDATE clients SET token_endpoint_auth_method = 'none' WHERE public_client;

CREATE TABLE client_assertion_jtis (
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    jti TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (client_id, jti)
);

CREATE INDEX idx_client_assertion_jtis_expires_at ON client_assertion_jtis (expires_at);

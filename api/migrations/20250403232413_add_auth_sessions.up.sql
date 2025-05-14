-- Add up migration script here
CREATE TABLE IF NOT EXISTS auth_sessions (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL,
    client_id UUID NOT NULL,
    redirect_uri VARCHAR(255) NOT NULL,
    response_type VARCHAR(255) NOT NULL,
    scope VARCHAR(255) NOT NULL,
    state VARCHAR(255),
    nonce VARCHAR(255),
    user_id UUID,
    code VARCHAR(255) NULL UNIQUE,
    authenticated BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,

    CONSTRAINT fk_realm
        FOREIGN KEY (realm_id)
        REFERENCES realms(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_client
        FOREIGN KEY (client_id)
        REFERENCES clients(id)
        ON DELETE CASCADE,

    CONSTRAINT fk_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE SET NULL
);

-- Create indexes for performance
CREATE INDEX idx_auth_sessions_user_id ON auth_sessions(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_auth_sessions_client_id ON auth_sessions(client_id);
CREATE INDEX idx_auth_sessions_realm_id ON auth_sessions(realm_id);
CREATE INDEX idx_auth_sessions_expires_at ON auth_sessions(expires_at);
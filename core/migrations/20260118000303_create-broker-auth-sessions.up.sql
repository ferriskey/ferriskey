-- Broker auth sessions for SSO state management
-- Stores OAuth state between login initiation and IdP callback
CREATE TABLE IF NOT EXISTS broker_auth_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    identity_provider_id UUID NOT NULL REFERENCES identity_providers(id) ON DELETE CASCADE,
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    redirect_uri VARCHAR(2048) NOT NULL,
    response_type VARCHAR(50) NOT NULL,
    scope VARCHAR(1024) NOT NULL DEFAULT '',
    state TEXT,
    nonce TEXT,
    broker_state VARCHAR(255) NOT NULL UNIQUE,
    code_verifier VARCHAR(255),
    auth_session_id UUID REFERENCES auth_sessions(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '10 minutes')
);

CREATE INDEX idx_broker_auth_sessions_broker_state ON broker_auth_sessions(broker_state);
CREATE INDEX idx_broker_auth_sessions_realm_id ON broker_auth_sessions(realm_id);
CREATE INDEX idx_broker_auth_sessions_expires_at ON broker_auth_sessions(expires_at);

-- Identity provider links (user <-> IdP identity mapping)
-- Stores the relationship between FerrisKey users and their external IdP identities
CREATE TABLE IF NOT EXISTS identity_provider_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    identity_provider_id UUID NOT NULL REFERENCES identity_providers(id) ON DELETE CASCADE,
    identity_provider_user_id VARCHAR(512) NOT NULL,
    identity_provider_username VARCHAR(255) NOT NULL,
    token TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One link per user per provider
    CONSTRAINT uq_idp_links_user_provider UNIQUE (user_id, identity_provider_id),
    -- One external ID per provider
    CONSTRAINT uq_idp_links_provider_external_id UNIQUE (identity_provider_id, identity_provider_user_id)
);

CREATE INDEX idx_idp_links_user_id ON identity_provider_links(user_id);
CREATE INDEX idx_idp_links_provider_id ON identity_provider_links(identity_provider_id);

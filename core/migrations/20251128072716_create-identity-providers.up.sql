-- Create identity_providers table
CREATE TABLE IF NOT EXISTS identity_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    alias VARCHAR(255) NOT NULL,
    provider_id VARCHAR(255) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    display_name VARCHAR(255),
    first_broker_login_flow_alias VARCHAR(255),
    post_broker_login_flow_alias VARCHAR(255),
    store_token BOOLEAN NOT NULL DEFAULT false,
    add_read_token_role_on_create BOOLEAN NOT NULL DEFAULT false,
    trust_email BOOLEAN NOT NULL DEFAULT false,
    link_only BOOLEAN NOT NULL DEFAULT false,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint: one alias per realm
    CONSTRAINT uq_identity_providers_realm_alias UNIQUE (realm_id, alias)
);

-- Index for performance
CREATE INDEX idx_identity_providers_realm_id ON identity_providers(realm_id);
CREATE INDEX idx_identity_providers_alias ON identity_providers(alias);
CREATE INDEX idx_identity_providers_provider_id ON identity_providers(provider_id);
CREATE INDEX idx_identity_providers_enabled ON identity_providers(enabled) WHERE enabled = true;

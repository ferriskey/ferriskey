CREATE TABLE user_federation_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    provider_type VARCHAR(50) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0,
    config JSONB NOT NULL,
    sync_enabled BOOLEAN NOT NULL DEFAULT false,
    sync_mode VARCHAR(20) NOT NULL DEFAULT 'import',
    sync_interval_minutes INTEGER,
    last_sync_at TIMESTAMPTZ,
    last_sync_status VARCHAR(50),
    last_sync_result JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(realm_id, name)
);

CREATE INDEX idx_federation_providers_realm_id ON user_federation_providers(realm_id);
CREATE INDEX idx_federation_providers_type ON user_federation_providers(provider_type);
CREATE INDEX idx_federation_providers_enabled ON user_federation_providers(enabled);
CREATE INDEX idx_federation_providers_realm_enabled_priority ON user_federation_providers(realm_id, enabled, priority);

CREATE TABLE user_federation_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES user_federation_providers(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    external_id VARCHAR(512) NOT NULL,
    external_username VARCHAR(255) NOT NULL,
    mapping_metadata JSONB,
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider_id, external_id),
    UNIQUE(user_id, provider_id)
);

CREATE INDEX idx_federation_mappings_provider_id ON user_federation_mappings(provider_id);
CREATE INDEX idx_federation_mappings_user_id ON user_federation_mappings(user_id);
CREATE INDEX idx_federation_mappings_external_id ON user_federation_mappings(external_id);

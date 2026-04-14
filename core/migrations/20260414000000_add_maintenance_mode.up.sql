-- Add maintenance mode columns to clients
ALTER TABLE clients ADD COLUMN maintenance_enabled BOOLEAN DEFAULT FALSE;
ALTER TABLE clients ADD COLUMN maintenance_reason TEXT;
ALTER TABLE clients ADD COLUMN maintenance_session_strategy VARCHAR(50);

-- Client-level maintenance whitelist
CREATE TABLE client_maintenance_whitelist (
    id UUID PRIMARY KEY,
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_whitelist_target CHECK (
        (user_id IS NOT NULL AND role_id IS NULL) OR
        (user_id IS NULL AND role_id IS NOT NULL)
    )
);

CREATE INDEX idx_client_maintenance_whitelist_client_id ON client_maintenance_whitelist(client_id);

-- Realm-level maintenance whitelist (inherited by all clients)
CREATE TABLE realm_maintenance_whitelist (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_realm_whitelist_target CHECK (
        (user_id IS NOT NULL AND role_id IS NULL) OR
        (user_id IS NULL AND role_id IS NOT NULL)
    )
);

CREATE INDEX idx_realm_maintenance_whitelist_realm_id ON realm_maintenance_whitelist(realm_id);

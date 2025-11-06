-- ============================================
-- Create table: ldap_providers
-- ============================================

CREATE TABLE IF NOT EXISTS ldap_providers (
    id                  UUID PRIMARY KEY,
    realm_id            UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    name                TEXT NOT NULL,
    url                 TEXT NOT NULL,
    bind_dn             TEXT NOT NULL,
    bind_password       TEXT NOT NULL,
    user_base_dn        TEXT NOT NULL,
    user_filter         TEXT NOT NULL DEFAULT '(objectClass=person)',
    username_attr       TEXT NOT NULL DEFAULT 'uid',
    email_attr          TEXT,
    display_name_attr   TEXT,
    enabled             BOOLEAN NOT NULL DEFAULT TRUE,
    created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- Indexes
-- ============================================

CREATE INDEX IF NOT EXISTS idx_ldap_providers_realm_id ON ldap_providers(realm_id);
CREATE INDEX IF NOT EXISTS idx_ldap_providers_name ON ldap_providers(name);

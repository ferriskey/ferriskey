-- Add up migration script here
CREATE TABLE organizations (
    id          UUID        PRIMARY KEY,
    realm_id    UUID        NOT NULL REFERENCES realms(id) ON DELETE CASCADE,

    name        VARCHAR(255) NOT NULL,
    alias       VARCHAR(255) NOT NULL,              -- URL-safe, unique par realm
    domain      VARCHAR(255),                        -- ex: acme.com (pour IdP redirect)
    redirect_url TEXT,                               -- URL de redirection post-auth
    description TEXT,

    enabled     BOOLEAN     NOT NULL DEFAULT TRUE,

    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_organization_alias_per_realm UNIQUE (realm_id, alias)
);

CREATE INDEX idx_organizations_realm_id ON organizations(realm_id);
CREATE INDEX idx_organizations_domain   ON organizations(domain) WHERE domain IS NOT NULL;

-- ============================================================

CREATE TABLE organization_attributes (
    id              UUID        PRIMARY KEY,
    organization_id UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    key             VARCHAR(255) NOT NULL,
    value           TEXT         NOT NULL,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_organization_attribute_key UNIQUE (organization_id, key)
);

CREATE INDEX idx_organization_attributes_org_id ON organization_attributes(organization_id);

-- ============================================================

CREATE TABLE organization_members (
    id              UUID        PRIMARY KEY,
    organization_id UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id         UUID        NOT NULL REFERENCES users(id)         ON DELETE CASCADE,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_organization_member UNIQUE (organization_id, user_id)
);

CREATE INDEX idx_organization_members_org_id  ON organization_members(organization_id);
CREATE INDEX idx_organization_members_user_id ON organization_members(user_id);

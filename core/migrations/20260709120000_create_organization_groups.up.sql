-- Hierarchical groups scoped to an organization (Keycloak-style).
-- Groups form a tree via parent_group_id (NULL = top-level). Membership is recursive.

CREATE TABLE organization_groups (
    id              UUID         PRIMARY KEY,
    organization_id UUID         NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    parent_group_id UUID         REFERENCES organization_groups(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_organization_groups_org_id ON organization_groups(organization_id);
CREATE INDEX idx_organization_groups_parent_id ON organization_groups(parent_group_id);

-- Sibling name uniqueness within the same parent (NULL parent collapsed to the nil UUID so
-- top-level siblings are also unique).
CREATE UNIQUE INDEX uq_organization_group_sibling_name ON organization_groups (
    organization_id,
    COALESCE(parent_group_id, '00000000-0000-0000-0000-000000000000'::uuid),
    name
);

CREATE TABLE organization_group_members (
    id         UUID        PRIMARY KEY,
    group_id   UUID        NOT NULL REFERENCES organization_groups(id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_organization_group_member UNIQUE (group_id, user_id)
);

CREATE INDEX idx_organization_group_members_group_id ON organization_group_members(group_id);
CREATE INDEX idx_organization_group_members_user_id ON organization_group_members(user_id);

CREATE TABLE organization_group_roles (
    id         UUID        PRIMARY KEY,
    group_id   UUID        NOT NULL REFERENCES organization_groups(id) ON DELETE CASCADE,
    role_id    UUID        NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_organization_group_role UNIQUE (group_id, role_id)
);

CREATE INDEX idx_organization_group_roles_group_id ON organization_group_roles(group_id);
CREATE INDEX idx_organization_group_roles_role_id ON organization_group_roles(role_id);

CREATE TABLE organization_group_attributes (
    id         UUID         PRIMARY KEY,
    group_id   UUID         NOT NULL REFERENCES organization_groups(id) ON DELETE CASCADE,
    key        VARCHAR(255) NOT NULL,
    value      TEXT         NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_organization_group_attribute_key UNIQUE (group_id, key)
);

CREATE INDEX idx_organization_group_attributes_group_id ON organization_group_attributes(group_id);

-- Roles attached directly to an organization member (Discord-style member-scoped roles).
-- Each row grants a realm or client role to a user *within the scope of one organization*.
-- Rows cascade away when the membership is removed or the role is deleted.

CREATE TABLE organization_member_roles (
    id                       UUID        PRIMARY KEY,
    organization_member_id   UUID        NOT NULL REFERENCES organization_members(id) ON DELETE CASCADE,
    role_id                  UUID        NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_organization_member_role UNIQUE (organization_member_id, role_id)
);

CREATE INDEX idx_organization_member_roles_member_id ON organization_member_roles(organization_member_id);
CREATE INDEX idx_organization_member_roles_role_id ON organization_member_roles(role_id);

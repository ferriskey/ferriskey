-- Add up migration script here
CREATE TABLE realm_portal_configs (
    id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    realm_id   UUID        NOT NULL UNIQUE REFERENCES realms (id) ON DELETE CASCADE,
    is_active  BOOLEAN     NOT NULL DEFAULT FALSE,
    layout     JSONB       NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_realm_portal_configs PRIMARY KEY (id)
);

CREATE INDEX idx_realm_portal_configs_realm_id ON realm_portal_configs (realm_id);

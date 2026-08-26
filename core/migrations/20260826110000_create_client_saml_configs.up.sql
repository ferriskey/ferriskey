CREATE TABLE IF NOT EXISTS client_saml_configs (
    client_id UUID PRIMARY KEY REFERENCES clients(id) ON DELETE CASCADE,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    sp_entity_id TEXT NOT NULL,
    acs_url TEXT NOT NULL,
    name_id_format TEXT NOT NULL,
    sign_assertions BOOLEAN NOT NULL,
    sign_documents BOOLEAN NOT NULL,
    want_authn_requests_signed BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_client_saml_configs_realm_id
ON client_saml_configs(realm_id);

CREATE UNIQUE INDEX IF NOT EXISTS uq_client_saml_configs_realm_id_sp_entity_id
ON client_saml_configs(realm_id, sp_entity_id);

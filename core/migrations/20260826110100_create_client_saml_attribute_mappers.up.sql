CREATE TABLE IF NOT EXISTS client_saml_attribute_mappers (
    id UUID PRIMARY KEY,
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    name_format TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_client_saml_attribute_mappers_client_id
ON client_saml_attribute_mappers(client_id);

CREATE UNIQUE INDEX IF NOT EXISTS uq_client_saml_attribute_mappers_client_id_name
ON client_saml_attribute_mappers(client_id, name);

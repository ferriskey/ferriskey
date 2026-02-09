CREATE TABLE client_scope_mappings (
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    client_scope_id UUID NOT NULL REFERENCES client_scopes(id) ON DELETE CASCADE,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    is_optional BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (client_id, scope_id)
);

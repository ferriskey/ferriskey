CREATE TABLE client_scope_protocol_mappers (
    id UUID PRIMARY KEY,
    client_scope_id UUID NOT NULL REFERENCES client_scopes(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    mapper_type VARCHAR(255) NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE client_scope_attributes (
    id UUID PRIMARY KEY,
    scope_id UUID NOT NULL REFERENCES client_scopes(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    value VARCHAR(2048),
    CONSTRAINT unique_attribute_per_scope UNIQUE (scope_id, name)
);

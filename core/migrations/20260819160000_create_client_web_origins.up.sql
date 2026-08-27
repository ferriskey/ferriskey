CREATE TABLE IF NOT EXISTS client_web_origins (
    id UUID PRIMARY KEY,
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    value TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_client_web_origins_client_id
ON client_web_origins(client_id);

CREATE UNIQUE INDEX IF NOT EXISTS uq_client_web_origins_client_id_value
ON client_web_origins(client_id, value);

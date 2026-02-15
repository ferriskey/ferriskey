CREATE TABLE IF NOT EXISTS post_logout_redirect_uris (
    id UUID PRIMARY KEY,
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    value TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_post_logout_redirect_uris_client_id
ON post_logout_redirect_uris(client_id);

CREATE INDEX idx_post_logout_redirect_uris_client_id_enabled
ON post_logout_redirect_uris(client_id, enabled);

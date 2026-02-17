-- Create magic_links table
CREATE TABLE magic_links (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    token_id UUID NOT NULL,
    token VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL
    -- used_at TIMESTAMP, -- Maybe in the future we'll need this for analytics or security concerns
    -- ip_address INET, -- same
);

CREATE UNIQUE INDEX IF NOT EXISTS magic_links_token_id_unique ON magic_links(token_id);
CREATE UNIQUE INDEX idx_magic_links_token ON magic_links(token);
CREATE INDEX idx_magic_links_user_realm ON magic_links(user_id, realm_id);

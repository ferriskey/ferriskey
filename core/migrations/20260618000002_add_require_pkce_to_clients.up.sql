-- Add per-client PKCE requirement flag (RFC 7636)
ALTER TABLE clients
    ADD COLUMN IF NOT EXISTS require_pkce BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS idx_access_tokens_revoked ON access_tokens (revoked);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_status ON refresh_tokens (status);

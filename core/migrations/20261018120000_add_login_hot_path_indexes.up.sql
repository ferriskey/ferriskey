CREATE INDEX IF NOT EXISTS idx_credentials_user_id ON credentials (user_id);
CREATE INDEX IF NOT EXISTS idx_realm_settings_realm_id ON realm_settings (realm_id);
CREATE INDEX IF NOT EXISTS idx_client_scope_protocol_mappers_client_scope_id ON client_scope_protocol_mappers (client_scope_id);
CREATE INDEX IF NOT EXISTS idx_redirect_uris_client_id ON redirect_uris (client_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at ON refresh_tokens (expires_at);
CREATE INDEX IF NOT EXISTS idx_access_tokens_expires_at ON access_tokens (expires_at);

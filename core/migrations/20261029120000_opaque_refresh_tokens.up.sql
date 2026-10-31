ALTER TABLE refresh_tokens
    ADD COLUMN token_hash BYTEA NULL,
    ADD COLUMN client_id TEXT NULL,
    ADD COLUMN scope TEXT NULL;

CREATE UNIQUE INDEX idx_refresh_tokens_token_hash
    ON refresh_tokens (token_hash)
    WHERE token_hash IS NOT NULL;

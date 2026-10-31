ALTER TABLE refresh_tokens
    ADD COLUMN previous_jti UUID NULL,
    ADD COLUMN previous_token_hash BYTEA NULL;

CREATE INDEX idx_refresh_tokens_previous_jti
    ON refresh_tokens (previous_jti) WHERE previous_jti IS NOT NULL;

CREATE INDEX idx_refresh_tokens_previous_token_hash
    ON refresh_tokens (previous_token_hash) WHERE previous_token_hash IS NOT NULL;

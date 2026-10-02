ALTER TABLE auth_sessions
    ADD COLUMN IF NOT EXISTS consent_token_hash TEXT NULL,
    ADD COLUMN IF NOT EXISTS prompt_consent BOOLEAN NOT NULL DEFAULT false;

CREATE UNIQUE INDEX IF NOT EXISTS idx_auth_sessions_consent_token_hash
    ON auth_sessions (consent_token_hash)
    WHERE consent_token_hash IS NOT NULL;

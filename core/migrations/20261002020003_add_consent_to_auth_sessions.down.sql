DROP INDEX IF EXISTS idx_auth_sessions_consent_token_hash;

ALTER TABLE auth_sessions
    DROP COLUMN IF EXISTS prompt_consent,
    DROP COLUMN IF EXISTS consent_token_hash;

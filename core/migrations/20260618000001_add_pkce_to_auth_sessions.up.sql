-- Add PKCE fields to auth_sessions (RFC 7636)
ALTER TABLE auth_sessions
    ADD COLUMN IF NOT EXISTS code_challenge TEXT,
    ADD COLUMN IF NOT EXISTS code_challenge_method VARCHAR(10);

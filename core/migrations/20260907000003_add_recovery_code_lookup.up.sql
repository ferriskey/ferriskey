-- Fast lookup key for recovery-code credentials. Storing the first 16 hex
-- chars of SHA-256(code) lets verification locate the single candidate row
-- instead of running Argon2id against every stored code, which prevents a
-- memory-hard DoS on the unauthenticated reset-password-with-recovery-code
-- endpoint (a single request could otherwise cost 10-16 x ~64MB of work).
ALTER TABLE credentials
    ADD COLUMN recovery_code_lookup TEXT;

CREATE INDEX idx_credentials_recovery_code_lookup
    ON credentials (user_id, recovery_code_lookup)
    WHERE recovery_code_lookup IS NOT NULL;

-- FK-007: revoking a session, logging out, disabling an account or changing a
-- password had no effect on tokens already issued. The `sid` claim was written into
-- every token and never read back, and nothing linked persisted state to live tokens.
--
-- Two things were missing to make revocation reachable from a session id.

-- Access tokens already carry `sid` inside their JSONB claims, so no column is
-- needed — only a way to find them without a sequential scan over every token ever
-- issued. Revocation is rare; the index exists so that it stays cheap as the table
-- grows.
CREATE INDEX idx_access_tokens_claims_sid ON access_tokens ((claims ->> 'sid'));

-- Refresh tokens carry no claims at all: the row holds jti, user, family and status.
-- Without this column a refresh token cannot be tied back to the session that
-- produced it, which is what let a revoked session keep minting fresh pairs through
-- rotation. Nullable because tokens predating this migration have no known session,
-- and because some flows legitimately establish none (client credentials).
ALTER TABLE refresh_tokens ADD COLUMN session_id UUID;

CREATE INDEX idx_refresh_tokens_session_id ON refresh_tokens (session_id);

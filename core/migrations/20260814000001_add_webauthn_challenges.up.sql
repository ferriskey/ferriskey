-- Self-service (Bearer) passkey registration keeps its WebAuthn challenge keyed
-- by the authenticated user id, instead of on an auth_session row. Persisting it
-- in a dedicated table (rather than a process-local map) keeps the challenge
-- consistent across instances and lets it expire by `expires_at`.
CREATE TABLE webauthn_challenges (
    user_id    UUID PRIMARY KEY,
    challenge  JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_webauthn_challenges_expires_at ON webauthn_challenges (expires_at);

-- Short-lived, single-use, user-bound step-up tokens minted by
-- /me/reauthenticate and required before sensitive self-service operations
-- (TOTP re-enrollment, passkey registration, credential deletion). Persisting
-- them (rather than a process-local map) keeps them consistent across
-- instances and lets them expire by `expires_at`.
CREATE TABLE step_up_tokens (
    id         UUID PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_step_up_tokens_user_id ON step_up_tokens (user_id);
CREATE INDEX idx_step_up_tokens_expires_at ON step_up_tokens (expires_at);

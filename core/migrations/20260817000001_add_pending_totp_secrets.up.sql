-- Pending TOTP secret for the self-service /me/totp/setup -> /me/totp/verify
-- flow. The secret is generated and persisted server-side at setup time and
-- consumed (single-use) at verify time, so a caller can never supply their own
-- secret or silently replace an existing authenticator. Keyed by user id so a
-- user has at most one pending secret.
CREATE TABLE pending_totp_secrets (
    user_id    UUID PRIMARY KEY,
    secret     TEXT NOT NULL,
    label      TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_pending_totp_secrets_expires_at ON pending_totp_secrets (expires_at);

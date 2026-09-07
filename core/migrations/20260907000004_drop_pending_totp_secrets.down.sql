-- Recreate the table exactly as the original 20260817000001 migration created
-- it, so rolling back the drop restores the historical schema.
CREATE TABLE pending_totp_secrets (
    user_id    UUID PRIMARY KEY,
    secret     TEXT NOT NULL,
    label      TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_pending_totp_secrets_expires_at ON pending_totp_secrets (expires_at);

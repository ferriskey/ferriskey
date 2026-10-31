DROP INDEX IF EXISTS idx_clients_previous_secret_expires_at;

ALTER TABLE clients
    DROP CONSTRAINT IF EXISTS clients_previous_secret_has_expiry;

ALTER TABLE clients
    DROP COLUMN IF EXISTS previous_secret_expires_at,
    DROP COLUMN IF EXISTS previous_secret;

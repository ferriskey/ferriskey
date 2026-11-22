ALTER TABLE clients
    ADD COLUMN previous_secret TEXT NULL,
    ADD COLUMN previous_secret_expires_at TIMESTAMP NULL;

ALTER TABLE clients
    ADD CONSTRAINT clients_previous_secret_has_expiry
    CHECK ((previous_secret IS NULL) = (previous_secret_expires_at IS NULL));

CREATE INDEX idx_clients_previous_secret_expires_at
    ON clients (previous_secret_expires_at) WHERE previous_secret_expires_at IS NOT NULL;

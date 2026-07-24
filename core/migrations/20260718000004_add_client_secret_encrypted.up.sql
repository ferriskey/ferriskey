ALTER TABLE clients
    ADD COLUMN IF NOT EXISTS secret_encrypted TEXT,
    ADD COLUMN IF NOT EXISTS secret_key_id VARCHAR(64);

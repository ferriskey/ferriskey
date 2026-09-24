ALTER TABLE clients
    ADD COLUMN IF NOT EXISTS token_exchange_enabled BOOLEAN NOT NULL DEFAULT false;

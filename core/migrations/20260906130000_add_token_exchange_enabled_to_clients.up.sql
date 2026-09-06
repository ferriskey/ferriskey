ALTER TABLE clients
    ADD COLUMN token_exchange_enabled BOOLEAN NOT NULL DEFAULT false;

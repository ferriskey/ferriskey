-- Alter auth_sessions table to change state and nonce columns from VARCHAR(255) to TEXT
ALTER TABLE webhooks ADD COLUMN headers JSONB NOT NULL DEFAULT '{}';

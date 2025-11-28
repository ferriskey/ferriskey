-- Alter auth_sessions table to change state and nonce columns from VARCHAR(255) to TEXT
ALTER TABLE auth_sessions
ALTER COLUMN state TYPE TEXT;

ALTER TABLE auth_sessions
ALTER COLUMN nonce TYPE TEXT;

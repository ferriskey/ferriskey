-- Revert auth_sessions table to change state and nonce columns back from TEXT to VARCHAR(255)
ALTER TABLE auth_sessions
ALTER COLUMN state TYPE VARCHAR(255);

ALTER TABLE auth_sessions
ALTER COLUMN nonce TYPE VARCHAR(255);

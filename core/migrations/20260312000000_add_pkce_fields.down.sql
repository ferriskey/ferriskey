-- Remove PKCE fields from auth_sessions table
ALTER TABLE auth_sessions
DROP COLUMN IF EXISTS code_challenge,
DROP COLUMN IF EXISTS code_challenge_method;

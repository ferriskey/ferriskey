-- Add PKCE fields to auth_sessions table for OAuth 2.1 compliance
ALTER TABLE auth_sessions
ADD COLUMN code_challenge TEXT,
ADD COLUMN code_challenge_method TEXT;

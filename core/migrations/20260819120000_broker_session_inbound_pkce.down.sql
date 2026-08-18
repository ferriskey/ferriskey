ALTER TABLE broker_auth_sessions DROP COLUMN IF EXISTS code_challenge_method;
ALTER TABLE broker_auth_sessions DROP COLUMN IF EXISTS code_challenge;

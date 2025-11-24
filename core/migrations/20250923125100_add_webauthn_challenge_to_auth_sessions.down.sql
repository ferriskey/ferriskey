ALTER TABLE auth_sessions
DROP COLUMN IF EXISTS webauthn_challenge,
DROP COLUMN IF EXISTS webauth_challenge_issued_at;

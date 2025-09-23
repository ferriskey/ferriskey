ALTER TABLE auth_sessions
DROP webauthn_challenge IF EXISTS,
DROP webauth_challenge_issued_at IF EXISTS;

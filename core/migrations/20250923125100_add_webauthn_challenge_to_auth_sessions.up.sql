ALTER TABLE auth_sessions
ADD COLUMN webauthn_challenge jsonb NULL,
ADD COLUMN webauthn_challenge_issued_at timestamp NULL;

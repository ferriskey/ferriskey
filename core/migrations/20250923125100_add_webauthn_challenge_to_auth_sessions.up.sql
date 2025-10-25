ALTER TABLE auth_sessions 
ADD webauthn_challenge jsonb NULL,
ADD webauthn_challenge_issued_at timestamp NULL;

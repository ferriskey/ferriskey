ALTER TABLE auth_sessions 
ADD webauthn_challenge text NULL,
ADD webauthn_challenge_issued_at timestamp NULL;

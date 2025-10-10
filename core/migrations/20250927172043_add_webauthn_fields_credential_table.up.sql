ALTER TABLE credentials 
ADD COLUMN webauthn_credential_id BYTEA NULL,
ADD COLUMN webauthn_public_key BYTEA NULL;

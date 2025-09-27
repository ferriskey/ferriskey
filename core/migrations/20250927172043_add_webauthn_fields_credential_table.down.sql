ALTER TABLE credentials 
DROP COLUMN IF EXISTS webauthn_credential_id,
DROP COLUMN IF EXISTS webauth_public_key;

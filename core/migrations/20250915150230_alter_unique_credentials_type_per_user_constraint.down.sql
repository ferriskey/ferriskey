ALTER TABLE credentials
DROP CONSTRAINT IF EXISTS unique_credential_type_per_user_id_idx;

ALTER TABLE credentials ADD CONSTRAINT unique_credential_type_per_user_id UNIQUE (user_id, credential_type);

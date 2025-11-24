DROP INDEX unique_credential_type_per_user_id_idx;

CREATE UNIQUE INDEX unique_credential_type_per_user_id_idx
ON credentials (user_id, credential_type)

WHERE credential_type <> 'recovery-code';

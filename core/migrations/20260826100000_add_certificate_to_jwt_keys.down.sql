DROP INDEX IF EXISTS jwt_keys_realm_id_key;

ALTER TABLE jwt_keys DROP COLUMN IF EXISTS certificate;

INSERT INTO jwt_keys (id, realm_id, private_key, public_key, created_at)
SELECT id, realm_id, private_key, public_key, created_at
FROM jwt_keys_superseded
ON CONFLICT (id) DO NOTHING;

DROP TABLE IF EXISTS jwt_keys_superseded;

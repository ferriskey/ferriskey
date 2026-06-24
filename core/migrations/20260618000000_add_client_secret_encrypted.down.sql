ALTER TABLE clients
    DROP COLUMN IF EXISTS secret_encrypted,
    DROP COLUMN IF EXISTS secret_key_id;

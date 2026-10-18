ALTER TABLE users DROP CONSTRAINT IF EXISTS users_federation_provider_id_fkey;
ALTER TABLE users DROP COLUMN IF EXISTS federation_provider_id;

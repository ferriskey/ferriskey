DROP TRIGGER IF EXISTS user_federation_mappings_sync_marker ON user_federation_mappings;
DROP FUNCTION IF EXISTS users_sync_federation_provider();
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_federation_provider_id_fkey;
ALTER TABLE users DROP COLUMN IF EXISTS federation_provider_id;

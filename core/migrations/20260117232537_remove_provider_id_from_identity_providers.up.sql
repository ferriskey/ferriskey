-- Remove provider_id column from identity_providers table
DROP INDEX IF EXISTS idx_identity_providers_provider_id;
ALTER TABLE identity_providers DROP COLUMN IF EXISTS provider_id;

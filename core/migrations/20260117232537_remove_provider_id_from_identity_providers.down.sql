-- Restore provider_id column to identity_providers table
ALTER TABLE identity_providers ADD COLUMN provider_id VARCHAR(255) NOT NULL DEFAULT '';
CREATE INDEX idx_identity_providers_provider_id ON identity_providers(provider_id);

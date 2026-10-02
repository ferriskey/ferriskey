ALTER TABLE realm_settings
    ADD COLUMN IF NOT EXISTS consent_ttl_days INTEGER NULL;

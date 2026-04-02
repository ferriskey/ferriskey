ALTER TABLE realm_settings
  ADD COLUMN email_verification_ttl_hours INTEGER NOT NULL DEFAULT 24;

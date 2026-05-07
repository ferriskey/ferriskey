ALTER TABLE realm_settings
  ADD COLUMN email_verification_enabled BOOLEAN NOT NULL DEFAULT false;

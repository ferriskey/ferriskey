ALTER TABLE realm_settings
  ADD COLUMN cimd_enabled BOOLEAN NOT NULL DEFAULT FALSE,
  ADD COLUMN dcr_enabled BOOLEAN NOT NULL DEFAULT FALSE,
  ADD COLUMN cimd_allowed_hosts TEXT[] NOT NULL DEFAULT '{}',
  ADD COLUMN allowed_resources TEXT[] NOT NULL DEFAULT '{}';

ALTER TABLE clients
  ADD COLUMN registration_source VARCHAR(32) NOT NULL DEFAULT 'admin'
    CHECK (registration_source IN ('admin', 'dynamic', 'metadata_document'));

ALTER TABLE auth_sessions
  ADD COLUMN issuer TEXT,
  ADD COLUMN resource TEXT;

ALTER TABLE refresh_tokens
  ADD COLUMN resource TEXT;

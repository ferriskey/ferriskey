ALTER TABLE refresh_tokens
  DROP COLUMN resource;

ALTER TABLE auth_sessions
  DROP COLUMN resource,
  DROP COLUMN issuer;

ALTER TABLE clients
  DROP COLUMN registration_source;

ALTER TABLE realm_settings
  DROP COLUMN allowed_resources,
  DROP COLUMN cimd_allowed_hosts,
  DROP COLUMN dcr_enabled,
  DROP COLUMN cimd_enabled;

-- Add up migration script here
CREATE TABLE realm_branding (
  id UUID PRIMARY KEY,
  realm_id UUID NOT NULL,
  config JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_realm_branding_realm
    FOREIGN KEY (realm_id)
    REFERENCES realms (id)
    ON DELETE CASCADE,

  CONSTRAINT uq_realm_branding_realm UNIQUE (realm_id)
);

CREATE INDEX idx_realm_branding_realm_id
  ON realm_branding (realm_id);

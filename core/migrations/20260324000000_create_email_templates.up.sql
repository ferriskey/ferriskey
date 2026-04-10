-- Add up migration script here
CREATE TABLE email_templates (
  id UUID PRIMARY KEY,
  realm_id UUID NOT NULL,
  name VARCHAR(255) NOT NULL,
  email_type VARCHAR(50) NOT NULL,
  structure JSONB NOT NULL,
  mjml TEXT NOT NULL,
  is_active BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_email_template_realm
    FOREIGN KEY (realm_id)
    REFERENCES realms (id)
    ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_email_templates_unique_active
  ON email_templates (realm_id, email_type)
  WHERE (is_active = true);

CREATE INDEX idx_email_templates_realm_type
  ON email_templates (realm_id, email_type);

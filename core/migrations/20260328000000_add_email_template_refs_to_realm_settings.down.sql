-- Add down migration script here
ALTER TABLE realm_settings
  DROP COLUMN reset_password_template_id,
  DROP COLUMN magic_link_template_id,
  DROP COLUMN email_verification_template_id;

ALTER TABLE email_templates ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT false;
CREATE UNIQUE INDEX idx_email_templates_unique_active
  ON email_templates (realm_id, email_type)
  WHERE (is_active = true);

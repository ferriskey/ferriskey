-- Add up migration script here
ALTER TABLE realm_settings
  ADD COLUMN reset_password_template_id UUID NULL
    CONSTRAINT fk_reset_password_template REFERENCES email_templates (id) ON DELETE SET NULL,
  ADD COLUMN magic_link_template_id UUID NULL
    CONSTRAINT fk_magic_link_template REFERENCES email_templates (id) ON DELETE SET NULL,
  ADD COLUMN email_verification_template_id UUID NULL
    CONSTRAINT fk_email_verification_template REFERENCES email_templates (id) ON DELETE SET NULL;

ALTER TABLE email_templates DROP COLUMN is_active;
DROP INDEX IF EXISTS idx_email_templates_unique_active;

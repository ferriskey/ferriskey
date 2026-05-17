-- Revert portal_themes per-page restructure.

ALTER TABLE realm_settings
  DROP CONSTRAINT fk_realm_settings_portal_theme,
  DROP COLUMN portal_theme_id;

DROP INDEX idx_portal_themes_layout_id;

ALTER TABLE portal_themes
  DROP CONSTRAINT fk_portal_themes_layout,
  DROP COLUMN page_verify_email,
  DROP COLUMN page_magic_link_verify,
  DROP COLUMN page_reset_password,
  DROP COLUMN page_forgot_password,
  DROP COLUMN page_totp,
  DROP COLUMN page_register,
  DROP COLUMN page_login,
  DROP COLUMN layout_id,
  DROP COLUMN name;

ALTER TABLE portal_themes RENAME COLUMN design_tokens TO config;

ALTER TABLE portal_themes
  ADD CONSTRAINT uq_portal_themes_realm UNIQUE (realm_id);

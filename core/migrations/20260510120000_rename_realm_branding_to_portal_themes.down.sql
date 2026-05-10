-- Revert portal_themes -> realm_branding.
ALTER TABLE portal_themes
  RENAME CONSTRAINT uq_portal_themes_realm TO uq_realm_branding_realm;

ALTER TABLE portal_themes
  RENAME CONSTRAINT fk_portal_themes_realm TO fk_realm_branding_realm;

ALTER INDEX idx_portal_themes_realm_id RENAME TO idx_realm_branding_realm_id;

ALTER TABLE portal_themes RENAME TO realm_branding;

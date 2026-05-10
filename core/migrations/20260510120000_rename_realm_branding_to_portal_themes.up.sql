-- Rename realm_branding -> portal_themes (table, indexes, constraints).
ALTER TABLE realm_branding RENAME TO portal_themes;

ALTER INDEX idx_realm_branding_realm_id RENAME TO idx_portal_themes_realm_id;

ALTER TABLE portal_themes
  RENAME CONSTRAINT fk_realm_branding_realm TO fk_portal_themes_realm;

ALTER TABLE portal_themes
  RENAME CONSTRAINT uq_realm_branding_realm TO uq_portal_themes_realm;
